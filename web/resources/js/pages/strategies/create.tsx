import { Head, useForm } from '@inertiajs/react';
import { useState } from 'react';
import {
    index,
    create,
    store,
} from '@/actions/App/Http/Controllers/StrategyController';
import StrategyEditorForm, {
    type StrategyFormData,
} from '@/components/strategy/strategy-editor-form';
import AppLayout from '@/layouts/app-layout';
import {
    createDefaultFormGraph,
    createDefaultNodeGraph,
} from '@/lib/strategy-defaults';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: index.url() },
    { title: 'Create', href: create.url() },
];

export default function StrategiesCreate() {
    const [defaultFormGraph] = useState(() => createDefaultFormGraph());
    const [defaultNodeGraph] = useState(() => createDefaultNodeGraph());
    const { data, setData, post, processing, errors } =
        useForm<StrategyFormData>({
            name: '',
            description: '',
            mode: 'form' as 'form' | 'node',
            graph: defaultFormGraph,
        });

    function handleSubmit(e: React.FormEvent): void {
        e.preventDefault();
        post(store.url());
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Create Strategy" />
            <div className="p-4 md:p-8">
                <div className="mb-8">
                    <h1 className="text-2xl font-bold tracking-tight">
                        Create Strategy
                    </h1>
                    <p className="mt-1 text-muted-foreground">
                        Define conditions, actions, and risk parameters for your
                        new strategy.
                    </p>
                </div>
                <StrategyEditorForm
                    data={data}
                    errors={errors}
                    processing={processing}
                    initialFormGraph={defaultFormGraph}
                    initialNodeGraph={defaultNodeGraph}
                    submitLabel="Create Strategy"
                    submitHint="You can edit this strategy later."
                    showAiBuilder
                    onSubmit={handleSubmit}
                    onChange={(nextData) => setData(nextData)}
                />
            </div>
        </AppLayout>
    );
}
