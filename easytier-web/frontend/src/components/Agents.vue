<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Button, InputText, Dialog, useToast } from 'primevue';
import { Api } from 'easytier-frontend-lib';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps<{
    api?: InstanceType<typeof Api.ApiClient>;
}>();

interface AgentEntry {
    id: number;
    name: string;
    virtual_ip: string;
    description: string | null;
    last_sync_at: string | null;
    last_sync_status: string;
    created_at: string;
    updated_at: string;
}

const agents = ref<AgentEntry[]>([]);
const showCreateDialog = ref(false);
const newName = ref('');
const newVirtualIp = ref('');
const newDescription = ref('');
const loading = ref(false);
const toast = useToast();
let refreshTimer: ReturnType<typeof setInterval> | null = null;

const loadAgents = async () => {
    loading.value = true;
    try {
        agents.value = (await props.api?.list_agents()) || [];
    } catch (e) {
        toast.add({ severity: 'error', summary: t('web.common.error'), detail: e, life: 3000 });
    } finally {
        loading.value = false;
    }
};

const createAgent = async () => {
    if (!newName.value || !newVirtualIp.value) {
        toast.add({ severity: 'warn', summary: t('web.common.error'), detail: 'Name and Virtual IP are required', life: 2000 });
        return;
    }
    try {
        await props.api?.create_agent(newName.value, newVirtualIp.value, newDescription.value || undefined);
        newName.value = '';
        newVirtualIp.value = '';
        newDescription.value = '';
        showCreateDialog.value = false;
        await loadAgents();
        toast.add({ severity: 'success', summary: t('web.common.success'), life: 2000 });
    } catch (e) {
        toast.add({ severity: 'error', summary: t('web.common.error'), detail: e, life: 3000 });
    } finally {
        loading.value = false;
    }
};

const deleteAgent = async (id: number) => {
    try {
        await props.api?.delete_agent(id);
        await loadAgents();
        toast.add({ severity: 'success', summary: t('web.common.success'), life: 2000 });
    } catch (e) {
        toast.add({ severity: 'error', summary: t('web.common.error'), detail: e, life: 3000 });
    }
};

const statusColor = (status: string) => {
    if (status === 'success') return 'text-green-600';
    if (status === 'failed') return 'text-red-600';
    return 'text-gray-400';
};

onMounted(() => {
    loadAgents();
    refreshTimer = setInterval(loadAgents, 30000);
});

onUnmounted(() => {
    if (refreshTimer) clearInterval(refreshTimer);
});
</script>

<template>
    <div class="flex flex-col gap-4">
        <div class="flex justify-between items-center">
            <h1 class="text-xl font-bold">{{ t('web.agents.title') }}</h1>
            <Button :label="t('web.agents.add')" icon="pi pi-plus" @click="showCreateDialog = true" />
        </div>

        <div v-if="agents.length === 0" class="text-center text-gray-500 py-8">
            {{ t('web.agents.empty') }}
        </div>

        <div v-else class="overflow-x-auto">
            <table class="w-full border-collapse">
                <thead>
                    <tr class="bg-gray-100 dark:bg-gray-700">
                        <th class="p-3 text-left">{{ t('web.agents.name') }}</th>
                        <th class="p-3 text-left">{{ t('web.agents.virtual_ip') }}</th>
                        <th class="p-3 text-left">{{ t('web.agents.description') }}</th>
                        <th class="p-3 text-left">{{ t('web.agents.last_sync_at') }}</th>
                        <th class="p-3 text-left">{{ t('web.agents.last_sync_status') }}</th>
                        <th class="p-3 text-left">{{ t('web.agents.actions') }}</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="agent in agents" :key="agent.id" class="border-b hover:bg-gray-50 dark:hover:bg-gray-800">
                        <td class="p-3 text-sm">{{ agent.name }}</td>
                        <td class="p-3 font-mono text-sm">{{ agent.virtual_ip }}</td>
                        <td class="p-3 text-sm">{{ agent.description || '-' }}</td>
                        <td class="p-3 text-sm">{{ agent.last_sync_at || '-' }}</td>
                        <td class="p-3 text-sm">
                            <span :class="statusColor(agent.last_sync_status)">{{ agent.last_sync_status }}</span>
                        </td>
                        <td class="p-3 text-sm">
                            <Button icon="pi pi-trash" severity="danger" text rounded
                                :title="t('web.agents.delete')" @click="deleteAgent(agent.id)" />
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <Dialog v-model:visible="showCreateDialog" :header="t('web.agents.add')" :modal="true" :closable="true">
            <div class="flex flex-col gap-4 p-4">
                <div>
                    <label class="block text-sm font-medium mb-1">{{ t('web.agents.name') }}</label>
                    <InputText v-model="newName" class="w-full" :placeholder="t('web.agents.name_placeholder')" />
                </div>
                <div>
                    <label class="block text-sm font-medium mb-1">{{ t('web.agents.virtual_ip') }}</label>
                    <InputText v-model="newVirtualIp" class="w-full" placeholder="10.0.210.253" />
                </div>
                <div>
                    <label class="block text-sm font-medium mb-1">{{ t('web.agents.description') }}</label>
                    <InputText v-model="newDescription" class="w-full" :placeholder="t('web.agents.description_placeholder')" />
                </div>
                <div class="flex justify-end gap-2 mt-4">
                    <Button :label="t('web.common.cancel')" severity="secondary" @click="showCreateDialog = false" />
                    <Button :label="t('web.common.save')" :loading="loading" @click="createAgent" />
                </div>
            </div>
        </Dialog>
    </div>
</template>
