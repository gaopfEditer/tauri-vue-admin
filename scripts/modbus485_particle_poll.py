#!/usr/bin/env python3
"""
工业粒子检测器 — Modbus RTU (RS-485) 现场联调脚本

硬件参数（已由 Modscan32 验证）:
  - 串口: COM3, 19200, 8-N-1
  - Slave ID: 1
  - 功能码 04 Input Register（不是 03 Holding）
  - 起始地址 0（Modscan 显示为 30001）
  - 读取长度 10~20

依赖:
  pip install pymodbus pyserial

用法示例:
  python scripts/modbus485_particle_poll.py
  python scripts/modbus485_particle_poll.py --port COM3 --count 16 --interval 1
  python scripts/modbus485_particle_poll.py --port /dev/ttyUSB0   # Linux
  python scripts/modbus485_particle_poll.py --port /dev/tty.usbserial-xxx  # macOS
"""

from __future__ import annotations

import argparse
import struct
import sys
import time
from datetime import datetime
from typing import Iterable, List, Optional, Sequence

try:
  from pymodbus.client import ModbusSerialClient
except ImportError:
  print("缺少依赖，请先执行: pip install pymodbus pyserial", file=sys.stderr)
  raise SystemExit(1)


# ---------------------------------------------------------------------------
# Float32 解析（两个连续 16 位寄存器）
# ---------------------------------------------------------------------------

def regs_to_float32_abcd(hi: int, lo: int) -> float:
  """
  ABCD 字序（大端 / big-endian）:
    寄存器0 = 高字 (AB), 寄存器1 = 低字 (CD)
    与常见 IEEE754 big-endian 一致。
  """
  raw = ((hi & 0xFFFF) << 16) | (lo & 0xFFFF)
  return struct.unpack(">f", struct.pack(">I", raw))[0]


def regs_to_float32_cdab(hi: int, lo: int) -> float:
  """
  CDAB 字序（字交换 / word-swap）:
    先把两个寄存器对调再按大端解浮点。
    即物理顺序 [reg0, reg1] 当作 [CD, AB]。
  """
  return regs_to_float32_abcd(lo, hi)


def decode_float_pairs(
  regs: Sequence[int],
  start: int = 0,
  order: str = "ABCD",
) -> Optional[float]:
  """从 regs[start], regs[start+1] 解 Float32；越界返回 None。"""
  if start + 1 >= len(regs):
    return None
  a, b = regs[start], regs[start + 1]
  if order.upper() == "CDAB":
    return regs_to_float32_cdab(a, b)
  return regs_to_float32_abcd(a, b)


# ---------------------------------------------------------------------------
# Modbus RTU 客户端
# ---------------------------------------------------------------------------

def make_client(port: str, baudrate: int, timeout: float) -> ModbusSerialClient:
  return ModbusSerialClient(
    port=port,
    baudrate=baudrate,
    bytesize=8,
    parity="N",
    stopbits=1,
    timeout=timeout,
  )


def read_input_registers(
  client: ModbusSerialClient,
  slave_id: int,
  address: int,
  count: int,
) -> List[int]:
  """
  功能码 04：读 Input Registers。
  兼容 pymodbus 3.x（device_id / slave 参数差异）。
  """
  kwargs = {"address": address, "count": count}
  try:
    rr = client.read_input_registers(device_id=slave_id, **kwargs)
  except TypeError:
    # 旧版 pymodbus
    rr = client.read_input_registers(slave=slave_id, **kwargs)

  if rr is None:
    raise RuntimeError("读寄存器无响应")
  if hasattr(rr, "isError") and rr.isError():
    raise RuntimeError(f"Modbus 异常: {rr}")
  regs = getattr(rr, "registers", None)
  if not regs:
    raise RuntimeError(f"无寄存器数据: {rr}")
  return [int(x) & 0xFFFF for x in regs]


def connect_with_retry(
  client: ModbusSerialClient,
  retries: int,
  retry_delay: float,
) -> None:
  last_err: Optional[BaseException] = None
  for attempt in range(1, retries + 1):
    try:
      ok = client.connect()
      if ok:
        print(f"[{now()}] 串口已连接")
        return
      last_err = RuntimeError("connect() 返回 False")
    except Exception as exc:  # noqa: BLE001 — 现场联调需捕获串口各类异常
      last_err = exc
    print(f"[{now()}] 连接失败 ({attempt}/{retries}): {last_err}")
    time.sleep(retry_delay)
  raise RuntimeError(f"无法建立串口连接: {last_err}")


def now() -> str:
  return datetime.now().strftime("%H:%M:%S")


def format_regs(regs: Iterable[int]) -> str:
  parts = [f"[{i}]={v}(0x{v:04X})" for i, v in enumerate(regs)]
  return " ".join(parts)


def poll_loop(
  port: str,
  baudrate: int,
  slave_id: int,
  address: int,
  count: int,
  interval: float,
  timeout: float,
  connect_retries: int,
  read_retries: int,
  float_order: str,
) -> None:
  client = make_client(port, baudrate, timeout)
  print(
    f"Modbus RTU 485 粒子联调 | port={port} {baudrate} 8N1 | "
    f"slave={slave_id} FC04 addr={address} count={count}"
  )

  try:
    connect_with_retry(client, connect_retries, retry_delay=1.0)
    cycle = 0
    while True:
      cycle += 1
      ok = False
      last_err: Optional[BaseException] = None
      for attempt in range(1, read_retries + 1):
        try:
          if not client.connected:
            connect_with_retry(client, connect_retries, retry_delay=1.0)
          regs = read_input_registers(client, slave_id, address, count)
          print(f"\n[{now()}] cycle#{cycle} 原始 Input Registers (u16):")
          print("  " + format_regs(regs))
          # 演示：把前两对寄存器按选定字序解成 Float32
          f0 = decode_float_pairs(regs, 0, float_order)
          f1 = decode_float_pairs(regs, 2, float_order)
          print(
            f"  Float32[{float_order}] reg0-1={f0!r}  reg2-3={f1!r}"
          )
          ok = True
          break
        except Exception as exc:  # noqa: BLE001
          last_err = exc
          print(
            f"[{now()}] 读取失败 ({attempt}/{read_retries}): {exc}"
          )
          try:
            client.close()
          except Exception:  # noqa: BLE001
            pass
          time.sleep(min(1.0 * attempt, 3.0))
      if not ok:
        print(f"[{now()}] 本周期放弃，稍后重试。最后错误: {last_err}")
      time.sleep(interval)
  except KeyboardInterrupt:
    print(f"\n[{now()}] 用户中断，退出")
  finally:
    try:
      client.close()
    except Exception:  # noqa: BLE001
      pass
    print(f"[{now()}] 串口已关闭")


def build_parser() -> argparse.ArgumentParser:
  p = argparse.ArgumentParser(
    description="工业粒子检测器 Modbus RTU (RS-485) 轮询联调"
  )
  p.add_argument("--port", default="COM3", help="串口，默认 COM3")
  p.add_argument("--baudrate", type=int, default=19200)
  p.add_argument("--slave-id", type=int, default=1)
  p.add_argument(
    "--address",
    type=int,
    default=0,
    help="Input Register 起始地址（0 = Modscan 30001）",
  )
  p.add_argument(
    "--count",
    type=int,
    default=16,
    help="读取长度，建议 10~20，默认 16",
  )
  p.add_argument("--interval", type=float, default=1.0, help="轮询间隔秒")
  p.add_argument("--timeout", type=float, default=1.0, help="串口超时秒")
  p.add_argument("--connect-retries", type=int, default=5)
  p.add_argument("--read-retries", type=int, default=3)
  p.add_argument(
    "--float-order",
    choices=("ABCD", "CDAB"),
    default="ABCD",
    help="连续两寄存器转 Float32 的字序",
  )
  return p


def main(argv: Optional[Sequence[str]] = None) -> int:
  args = build_parser().parse_args(argv)
  if not (10 <= args.count <= 20):
    print("警告: --count 建议在 10~20（仍继续执行）", file=sys.stderr)

  # 自检解析函数
  assert abs(regs_to_float32_abcd(0x3F80, 0x0000) - 1.0) < 1e-6
  assert abs(regs_to_float32_cdab(0x0000, 0x3F80) - 1.0) < 1e-6

  poll_loop(
    port=args.port,
    baudrate=args.baudrate,
    slave_id=args.slave_id,
    address=args.address,
    count=args.count,
    interval=args.interval,
    timeout=args.timeout,
    connect_retries=args.connect_retries,
    read_retries=args.read_retries,
    float_order=args.float_order,
  )
  return 0


if __name__ == "__main__":
  raise SystemExit(main())
