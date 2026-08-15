<template>
  <n-card title="角色管理" :bordered="false" class="rounded-16px shadow-sm">
    <n-space class="pb-12px" justify="space-between">
      <n-space />
      <n-space align="center" :size="18">
        <n-button size="small" type="primary" @click="getTableData">
          <icon-mdi-refresh class="mr-4px text-16px" :class="{ 'animate-spin': loading }" />
          刷新表格
        </n-button>
      </n-space>
    </n-space>
    <n-data-table :columns="columns" :data="tableData" :loading="loading" :pagination="pagination" />
  </n-card>
</template>

<script setup lang="tsx">
import { reactive, ref } from 'vue';
import type { Ref } from 'vue';
import { NTag } from 'naive-ui';
import type { DataTableColumns, PaginationProps } from 'naive-ui';
import { fetchRoleList } from '@/service';
import { useLoading } from '@/hooks';

const { loading, startLoading, endLoading } = useLoading(false);
const tableData = ref<RoleManagement.Role[]>([]);

async function getTableData() {
  startLoading();
  const { data } = await fetchRoleList();
  tableData.value = data ?? [];
  endLoading();
}

const columns: Ref<DataTableColumns<RoleManagement.Role>> = ref([
  { key: 'index', title: '序号', align: 'center', width: 80 },
  { key: 'roleCode', title: '角色编码', align: 'center' },
  { key: 'roleName', title: '角色名称', align: 'center' },
  {
    key: 'homeRoute',
    title: '首页路由',
    align: 'center',
    render: row => <NTag type="info">{row.homeRoute}</NTag>
  },
  {
    key: 'description',
    title: '描述',
    align: 'center',
    ellipsis: { tooltip: true },
    render: row => row.description || '-'
  }
]) as Ref<DataTableColumns<RoleManagement.Role>>;

const pagination = reactive<PaginationProps>({
  page: 1,
  pageSize: 10,
  showSizePicker: true,
  pageSizes: [10, 15, 20],
  onChange: (page: number) => {
    pagination.page = page;
  },
  onUpdatePageSize: (pageSize: number) => {
    pagination.pageSize = pageSize;
    pagination.page = 1;
  }
});

getTableData();
</script>

<style scoped></style>
