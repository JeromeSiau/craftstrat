import { Head, useForm } from '@inertiajs/react';
import { index, show, edit, update } from '@/actions/App/Http/Controllers/StrategyController';
import InputError from '@/components/input-error';
import FormBuilder from '@/components/strategy/form-builder';
import NodeEditor from '@/components/strategy/node-editor';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import AppLayout from '@/layouts/app-layout';
import type { BreadcrumbItem } from '@/types';
import type { FormModeGraph, NodeModeGraph, Strategy } from '@/types/models';

interface Props {
    strategy: Strategy;
}

export default function StrategiesEdit({ strategy }: Props) {
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Strategies', href: index.url() },
        { title: strategy.name, href: show.url(strategy.id) },
        { title: 'Edit', href: edit.url(strategy.id) },
    ];

    const { data, setData, put, processing, errors } = useForm({
        name: strategy.name,
        description: strategy.description ?? '',
        mode: strategy.mode as 'form' | 'node',
        graph: strategy.graph as FormModeGraph | NodeModeGraph,
    });

    function handleTabChange(tab: string): void {
        if (tab === 'form') {
            setData({ ...data, mode: 'form', graph: strategy.mode === 'form' ? (strategy.graph as FormModeGraph) : data.graph });
        } else {
            setData({ ...data, mode: 'node', graph: strategy.mode === 'node' ? (strategy.graph as NodeModeGraph) : data.graph });
        }
    }

    function handleSubmit(e: React.FormEvent): void {
        e.preventDefault();
        put(update.url(strategy.id));
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={`Edit ${strategy.name}`} />
            <div className="p-4 md:p-8">
                <div className="mb-8">
                    <h1 className="text-2xl font-bold tracking-tight">Edit Strategy</h1>
                    <p className="mt-1 text-muted-foreground">
                        Update conditions, actions, and risk parameters for your strategy.
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

                    <Tabs defaultValue={strategy.mode} onValueChange={handleTabChange}>
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
                                Save Changes
                            </Button>
                            <p className="text-sm text-muted-foreground">
                                Changes will take effect on next strategy evaluation.
                            </p>
                        </div>
                    </div>
                </form>
            </div>
        </AppLayout>
    );
}
