<template>
  <n-modal v-model:show="modalVisible" preset="card" :title="title" class="w-700px">
    <n-form ref="formRef" label-placement="left" :label-width="80" :model="formModel" :rules="rules">
      <n-grid :cols="24" :x-gap="18">
        <n-form-item-grid-item :span="12" label="用户名" path="userName">
          <n-input v-model:value="formModel.userName" placeholder="登录账号" />
        </n-form-item-grid-item>
        <n-form-item-grid-item :span="12" label="密码" path="password">
          <n-input
            v-model:value="formModel.password"
            type="password"
            show-password-on="click"
            :placeholder="passwordPlaceholder"
          />
        </n-form-item-grid-item>
        <n-form-item-grid-item v-if="isAdd" :span="12" label="角色" path="roleId">
          <n-select v-model:value="formModel.roleId" :options="roleOptions" placeholder="请选择角色" />
        </n-form-item-grid-item>
        <n-form-item-grid-item :span="12" label="年龄" path="age">
          <n-input-number v-model:value="formModel.age" clearable class="w-full" />
        </n-form-item-grid-item>
        <n-form-item-grid-item :span="12" label="性别" path="gender">
          <n-radio-group v-model:value="formModel.gender">
            <n-radio v-for="item in genderOptions" :key="item.value" :value="item.value">{{ item.label }}</n-radio>
          </n-radio-group>
        </n-form-item-grid-item>
        <n-form-item-grid-item :span="12" label="手机号" path="phone">
          <n-input v-model:value="formModel.phone" />
        </n-form-item-grid-item>
        <n-form-item-grid-item :span="12" label="邮箱" path="email">
          <n-input v-model:value="formModel.email" />
        </n-form-item-grid-item>
        <n-form-item-grid-item :span="12" label="状态" path="userStatus">
          <n-select v-model:value="formModel.userStatus" :options="userStatusOptions" />
        </n-form-item-grid-item>
      </n-grid>
      <n-space class="w-full pt-16px" :size="24" justify="end">
        <n-button class="w-72px" @click="closeModal">取消</n-button>
        <n-button class="w-72px" type="primary" :loading="submitting" @click="handleSubmit">确定</n-button>
      </n-space>
    </n-form>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, reactive, watch, onMounted } from 'vue';
import type { FormInst, FormItemRule, SelectOption } from 'naive-ui';
import { createUser, fetchRoleList, updateUser } from '@/service';
import { formRules, createRequiredFormRule } from '@/utils';
import { genderOptions, userStatusOptions } from '@/constants';

export interface Props {
  visible: boolean;
  type?: 'add' | 'edit';
  editData?: UserManagement.User | null;
}

export type ModalType = NonNullable<Props['type']>;

defineOptions({ name: 'TableActionModal' });

const props = withDefaults(defineProps<Props>(), {
  type: 'add',
  editData: null
});

interface Emits {
  (e: 'update:visible', visible: boolean): void;
  (e: 'success'): void;
}

const emit = defineEmits<Emits>();

const isAdd = computed(() => props.type === 'add');
const submitting = ref(false);
const roleOptions = ref<SelectOption[]>([]);

const modalVisible = computed({
  get() {
    return props.visible;
  },
  set(visible) {
    emit('update:visible', visible);
  }
});

const closeModal = () => {
  modalVisible.value = false;
};

const title = computed(() => (isAdd.value ? '添加用户' : '编辑用户'));
const passwordPlaceholder = computed(() => (isAdd.value ? '请设置登录密码' : '留空则不修改密码'));

const formRef = ref<HTMLElement & FormInst>();

type FormModel = {
  userName: string;
  password: string;
  roleId: number | null;
  age: number | null;
  gender: UserManagement.GenderKey | null;
  phone: string;
  email: string | null;
  userStatus: UserManagement.UserStatusKey | null;
};

const formModel = reactive<FormModel>(createDefaultFormModel());

const rules = computed<Record<keyof FormModel, FormItemRule | FormItemRule[]>>(() => ({
  userName: createRequiredFormRule('请输入用户名'),
  password: isAdd.value
    ? [createRequiredFormRule('请设置登录密码'), { min: 6, message: '密码至少 6 位', trigger: 'input' }]
    : [
        {
          validator: (_rule, value: string) => {
            if (!value || value.length >= 6) return true;
            return new Error('密码至少 6 位');
          },
          trigger: 'input'
        }
      ],
  roleId: isAdd.value ? createRequiredFormRule('请选择角色') : [],
  age: createRequiredFormRule('请输入年龄'),
  gender: createRequiredFormRule('请选择性别'),
  phone: formRules.phone,
  email: formRules.email,
  userStatus: createRequiredFormRule('请选择用户状态')
}));

function createDefaultFormModel(): FormModel {
  return {
    userName: '',
    password: '',
    roleId: null,
    age: null,
    gender: null,
    phone: '',
    email: null,
    userStatus: '1'
  };
}

function handleUpdateFormModel(model: Partial<FormModel>) {
  Object.assign(formModel, model);
}

function handleUpdateFormModelByModalType() {
  if (isAdd.value) {
    handleUpdateFormModel(createDefaultFormModel());
    return;
  }
  if (props.editData) {
    handleUpdateFormModel({
      userName: props.editData.userName ?? '',
      password: '',
      roleId: null,
      age: props.editData.age,
      gender: props.editData.gender,
      phone: props.editData.phone,
      email: props.editData.email,
      userStatus: props.editData.userStatus
    });
  }
}

async function loadRoles() {
  const { data } = await fetchRoleList();
  roleOptions.value =
    data?.map(item => ({
      label: `${item.roleName} (${item.roleCode})`,
      value: item.id
    })) ?? [];
}

async function handleSubmit() {
  await formRef.value?.validate();
  submitting.value = true;

  const payload: ApiUserManagement.UserUpsert = {
    userName: formModel.userName,
    age: formModel.age,
    gender: formModel.gender ?? undefined,
    phone: formModel.phone,
    email: formModel.email,
    userStatus: formModel.userStatus ?? undefined
  };

  if (formModel.password) {
    payload.password = formModel.password;
  }
  if (isAdd.value && formModel.roleId) {
    payload.roleId = formModel.roleId;
  }

  const result = isAdd.value ? await createUser(payload) : await updateUser(props.editData!.id, payload);

  submitting.value = false;

  if (result.error) {
    window.$message?.error(result.error.msg || '操作失败');
    return;
  }

  window.$message?.success(isAdd.value ? '新增成功' : '更新成功');
  emit('success');
  closeModal();
}

onMounted(loadRoles);

watch(
  () => props.visible,
  newValue => {
    if (newValue) {
      handleUpdateFormModelByModalType();
    }
  }
);
</script>

<style scoped></style>
