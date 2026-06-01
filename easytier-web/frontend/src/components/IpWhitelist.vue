<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Button, InputText, Dialog, useToast } from 'primevue';
import { Api } from 'easytier-frontend-lib';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps<{
    api?: InstanceType<typeof Api.ApiClient>;
}>();

interface WhitelistEntry {
    id: number;
    ip: string;
    comment: string | null;
    hostname: string | null;
    created_by: string;
    created_at: string;
}

const entries = ref<WhitelistEntry[]>([]);
const showCreateDialog = ref(false);
const newIp = ref('');
const newComment = ref('');
const loading = ref(false);
const toast = useToast();

const loadEntries = async () => {
    loading.value = true;
    try {
        entries.value = (await props.api?.list_ip_whitelist()) || [];
    } catch (e) {
        toast.add({ severity: 'error', summary: t('web.common.error'), detail: e, life: 3000 });
    } finally {
        loading.value = false;
    }
};

const createEntry = async () => {
    if (!newIp.value) {
        toast.add({ severity: 'warn', summary: t('web.common.error'), detail: 'IP is required', life: 2000 });
        return;
    }
    try {
        await props.api?.create_ip_whitelist_entry(newIp.value, newComment.value);
        newIp.value = '';
        newComment.value = '';
        showCreateDialog.value = false;
        await loadEntries();
        toast.add({ severity: 'success', summary: t('web.common.success'), life: 2000 });
    } catch (e) {
        toast.add({ severity: 'error', summary: t('web.common.error'), detail: e, life: 3000 });
    } finally {
        loading.value = false;
    }
};

const deleteEntry = async (id: number) => {
    try {
        await props.api?.delete_ip_whitelist_entry(id);
        await loadEntries();
        toast.add({ severity: 'success', summary: t('web.common.success'), life: 2000 });
    } catch (e) {
        toast.add({ severity: 'error', summary: t('web.common.error'), detail: e, life: 3000 });
    }
};

const unbindHostname = async (id: number) => {
    try {
        await props.api?.unbind_ip_whitelist_hostname(id);
        await loadEntries();
        toast.add({ severity: 'success', summary: t('web.common.success'), life: 2000 });
    } catch (e) {
        toast.add({ severity: 'error', summary: t('web.common.error'), detail: e, life: 3000 });
    }
};

onMounted(() => {
    loadEntries();
});
</script>

<template>
    <div class="flex flex-col gap-4">
        <div class="flex justify-between items-center">
            <h1 class="text-xl font-bold">{{ t('web.ipwhitelist.title') }}</h1>
            <Button :label="t('web.ipwhitelist.add')" icon="pi pi-plus" @click="showCreateDialog = true" />
        </div>

        <div v-if="entries.length === 0" class="text-center text-gray-500 py-8">
            {{ t('web.ipwhitelist.empty') }}
        </div>

        <div v-else class="overflow-x-auto">
            <table class="w-full border-collapse">
                <thead>
                    <tr class="bg-gray-100 dark:bg-gray-700">
                        <th class="p-3 text-left">{{ t('web.ipwhitelist.ip') }}</th>
                        <th class="p-3 text-left">{{ t('web.ipwhitelist.comment') }}</th>
                        <th class="p-3 text-left">{{ t('web.ipwhitelist.hostname') }}</th>
                        <th class="p-3 text-left">{{ t('web.ipwhitelist.created_by') }}</th>
                        <th class="p-3 text-left">{{ t('web.ipwhitelist.created_at') }}</th>
                        <th class="p-3 text-left">{{ t('web.ipwhitelist.actions') }}</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="entry in entries" :key="entry.id" class="border-b hover:bg-gray-50 dark:hover:bg-gray-800">
                        <td class="p-3 font-mono text-sm">{{ entry.ip }}</td>
                        <td class="p-3 text-sm">{{ entry.comment || '-' }}</td>
                        <td class="p-3 text-sm">{{ entry.hostname || '-' }}</td>
                        <td class="p-3 text-sm">{{ entry.created_by }}</td>
                        <td class="p-3 text-sm">{{ entry.created_at }}</td>
                        <td class="p-3 text-sm flex gap-2">
                            <Button v-if="entry.hostname" icon="pi pi-times" severity="warning" text rounded
                                :title="t('web.ipwhitelist.unbind')" @click="unbindHostname(entry.id)" />
                            <Button icon="pi pi-trash" severity="danger" text rounded
                                :title="t('web.ipwhitelist.delete')" @click="deleteEntry(entry.id)" />
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <Dialog v-model:visible="showCreateDialog" :header="t('web.ipwhitelist.add')" :modal="true" :closable="true">
            <div class="flex flex-col gap-4 p-4">
                <div>
                    <label class="block text-sm font-medium mb-1">{{ t('web.ipwhitelist.ip') }}</label>
                    <InputText v-model="newIp" class="w-full" :placeholder="t('web.ipwhitelist.ip_placeholder')" />
                </div>
                <div>
                    <label class="block text-sm font-medium mb-1">{{ t('web.ipwhitelist.comment') }}</label>
                    <InputText v-model="newComment" class="w-full" :placeholder="t('web.ipwhitelist.comment_placeholder')" />
                </div>
                <div class="flex justify-end gap-2 mt-4">
                    <Button :label="t('web.common.cancel')" severity="secondary" @click="showCreateDialog = false" />
                    <Button :label="t('web.common.save')" :loading="loading" @click="createEntry" />
                </div>
            </div>
        </Dialog>
    </div>
</template>
