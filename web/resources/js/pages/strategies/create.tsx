import { Head, useForm } from '@inertiajs/react';
import { index, create, store } from '@/actions/App/Http/Controllers/StrategyController';
import InputError from '@/components/input-error';
import AiBuilder from '@/components/strategy/ai-builder';
import FormBuilder from '@/components/strategy/form-builder';
import NodeEditor from '@/components/strategy/node-editor';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import AppLayout from '@/layouts/app-layout';
import { uid } from '@/lib/formatters';
import type { BreadcrumbItem } from '@/types';
import type { FormModeGraph, NodeModeGraph } from '@/types/models';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: index.url() },
    { title: 'Create', href: create.url() },
];

const defaultFormGraph: FormModeGraph = {
    mode: 'form',
    conditions: [
        {
            id: uid(),
            type: 'AND',
            rules: [{ id: uid(), indicator: 'abs_move_pct', operator: '>', value: 3.0 }],
        },
    ],
    action: {
        signal: 'buy',
        outcome: 'UP',
        size_mode: 'fixed',
        size_usdc: 50,
        order_type: 'market',
    },
    risk: {
        stoploss_pct: null,
        take_profit_pct: null,
        max_position_usdc: 200,
        max_trades_per_slot: 1,
        daily_loss_limit_usdc: null,
        cooldown_seconds: null,
        prevent_duplicates: false,
    },
};

const defaultNodeGraph: NodeModeGraph = {
    mode: 'node',
    nodes: [
        { id: 'n1', type: 'input', data: { field: 'abs_move_pct' }, position: { x: 50, y: 100 } },
        { id: 'n2', type: 'comparator', data: { operator: '>', value: 3.0 }, position: { x: 300, y: 100 } },
        { id: 'n3', type: 'action', data: { signal: 'buy', outcome: 'UP', size_usdc: 50 }, position: { x: 550, y: 100 } },
    ],
    edges: [
        { source: 'n1', target: 'n2' },
        { source: 'n2', target: 'n3' },
    ],
};

export default function StrategiesCreate() {
    const { data, setData, post, processing, errors } = useForm({
        name: '',
        description: '',
        mode: 'form' as 'form' | 'node',
        graph: defaultFormGraph as FormModeGraph | NodeModeGraph,
    });

    function handleTabChange(tab: string): void {
        if (tab === 'form') {
            setData({ ...data, mode: 'form', graph: defaultFormGraph });
        } else {
            setData({ ...data, mode: 'node', graph: defaultNodeGraph });
        }
    }

    function handleSubmit(e: React.FormEvent): void {
        e.preventDefault();
        post(store.url());
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Create Strategy" />
            <div className="p-4 md:p-8">
                <div className="mb-8">
                    <h1 className="text-2xl font-bold tracking-tight">Create Strategy</h1>
                    <p className="mt-1 text-muted-foreground">
                        Define conditions, actions, and risk parameters for your new strategy.
                    </p>
                </div>
                <form onSubmit={handleSubmit} className="space-y-8">
                    <div className="grid gap-6 sm:grid-cols-2">
                        <div className="space-y-2">
                            <Label htmlFor="name">Name</Label>
                            <Input
                                id="name"
                                value={data.name}
                                onChange={(e) => setData('name', e.target.value)}
                                placeholder="e.g. BTC Momentum Long"
                            />
                            <InputError message={errors.name} />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="description">Description</Label>
                            <Input
                                id="description"
                                value={data.description}
                                onChange={(e) => setData('description', e.target.value)}
                                placeholder="Describe what this strategy does..."
                            />
                        </div>
                    </div>

                    <AiBuilder
                        onGenerated={(graph) => setData({ ...data, mode: 'form', graph })}
                    />

                    <Tabs defaultValue="form" onValueChange={handleTabChange}>
                        <TabsList>
                            <TabsTrigger value="form">Form Builder</TabsTrigger>
                            <TabsTrigger value="node">Node Editor</TabsTrigger>
                        </TabsList>
                        <TabsContent value="form" className="mt-6">
                            <FormBuilder
                                graph={data.graph as FormModeGraph}
                                onChange={(graph) => setData('graph', graph)}
                            />
                        </TabsContent>
                        <TabsContent value="node" className="mt-6">
                            <NodeEditor
                                graph={data.graph as NodeModeGraph}
                                onChange={(graph) => setData('graph', graph)}
                            />
                        </TabsContent>
                    </Tabs>

                    <InputError message={errors.graph} />

                    <div className="sticky bottom-0 z-10 -mx-4 border-t bg-background/80 px-4 py-4 backdrop-blur-sm md:-mx-8 md:px-8">
                        <div className="flex items-center gap-4">
                            <Button type="submit" size="lg" disabled={processing}>
                                Create Strategy
                            </Button>
                            <p className="text-sm text-muted-foreground">
                                You can edit this strategy later.
                            </p>
                        </div>
                    </div>
                </form>
            </div>
        </AppLayout>
    );
}
