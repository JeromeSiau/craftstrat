import { Head, useForm } from '@inertiajs/react';
import { useState } from 'react';
import {
    index,
    show,
    edit,
    update,
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
import type { FormModeGraph, NodeModeGraph, Strategy } from '@/types/models';

interface Props {
    strategy: Strategy;
}

export default function StrategiesEdit({ strategy }: Props) {
    const [defaultFormGraph] = useState(() => createDefaultFormGraph());
    const [defaultNodeGraph] = useState(() => createDefaultNodeGraph());
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Strategies', href: index.url() },
        { title: strategy.name, href: show.url(strategy.id) },
        { title: 'Edit', href: edit.url(strategy.id) },
    ];

    const { data, setData, put, processing, errors } =
        useForm<StrategyFormData>({
            name: strategy.name,
            description: strategy.description ?? '',
            mode: strategy.mode as 'form' | 'node',
            graph: strategy.graph as FormModeGraph | NodeModeGraph,
        });

    function handleSubmit(e: React.FormEvent): void {
        e.preventDefault();
        put(update.url(strategy.id));
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={`Edit ${strategy.name}`} />
            <div className="p-4 md:p-8">
                <div className="mb-8">
                    <h1 className="text-2xl font-bold tracking-tight">
                        Edit Strategy
                    </h1>
                    <p className="mt-1 text-muted-foreground">
                        Update conditions, actions, and risk parameters for your
                        strategy.
                    </p>
                </div>
                <StrategyEditorForm
                    data={data}
                    errors={errors}
                    processing={processing}
                    initialFormGraph={
                        strategy.mode === 'form'
                            ? (strategy.graph as FormModeGraph)
                            : defaultFormGraph
                    }
                    initialNodeGraph={
                        strategy.mode === 'node'
                            ? (strategy.graph as NodeModeGraph)
                            : defaultNodeGraph
                    }
                    submitLabel="Save Changes"
                    submitHint="Changes will take effect on next strategy evaluation."
                    onSubmit={handleSubmit}
                    onChange={(nextData) => setData(nextData)}
                />
            </div>
        </AppLayout>
    );
}
