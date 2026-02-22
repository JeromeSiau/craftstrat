import { Head, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import FormBuilder from '@/components/strategy/form-builder';
import type { BreadcrumbItem } from '@/types';
import type { FormModeGraph } from '@/types/models';
import { index, create, store } from '@/actions/App/Http/Controllers/StrategyController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: index.url() },
    { title: 'Create', href: create.url() },
];

const defaultGraph: FormModeGraph = {
    mode: 'form',
    conditions: [
        {
            type: 'AND',
            rules: [{ indicator: 'abs_move_pct', operator: '>', value: 3.0 }],
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
        stoploss_pct: 30,
        take_profit_pct: 80,
        max_position_usdc: 200,
        max_trades_per_slot: 1,
    },
};

export default function StrategiesCreate() {
    const { data, setData, post, processing, errors } = useForm({
        name: '',
        description: '',
        mode: 'form',
        graph: defaultGraph as FormModeGraph,
    });

    function handleSubmit(e: React.FormEvent): void {
        e.preventDefault();
        post(store.url());
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Create Strategy" />
            <div className="mx-auto max-w-3xl p-6">
                <h1 className="mb-6 text-2xl font-bold">Create Strategy</h1>
                <form onSubmit={handleSubmit} className="space-y-6">
                    <div className="grid gap-4 sm:grid-cols-2">
                        <div className="space-y-2">
                            <Label htmlFor="name">Name</Label>
                            <Input
                                id="name"
                                value={data.name}
                                onChange={(e) => setData('name', e.target.value)}
                            />
                            {errors.name && (
                                <p className="text-sm text-red-500">{errors.name}</p>
                            )}
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="description">Description</Label>
                            <Textarea
                                id="description"
                                value={data.description}
                                onChange={(e) => setData('description', e.target.value)}
                                rows={1}
                            />
                        </div>
                    </div>

                    <Tabs defaultValue="form">
                        <TabsList>
                            <TabsTrigger value="form">Form Builder</TabsTrigger>
                            <TabsTrigger value="node" disabled>
                                Node Editor (coming soon)
                            </TabsTrigger>
                        </TabsList>
                        <TabsContent value="form" className="mt-4">
                            <FormBuilder
                                graph={data.graph}
                                onChange={(graph) => setData('graph', graph)}
                            />
                        </TabsContent>
                        <TabsContent value="node">
                            <p className="py-8 text-center text-sm text-muted-foreground">
                                Node editor is not yet available.
                            </p>
                        </TabsContent>
                    </Tabs>

                    {errors.graph && (
                        <p className="text-sm text-red-500">{errors.graph}</p>
                    )}

                    <Button type="submit" disabled={processing}>
                        Create Strategy
                    </Button>
                </form>
            </div>
        </AppLayout>
    );
}
