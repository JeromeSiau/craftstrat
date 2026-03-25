import InputError from '@/components/input-error';
import AiBuilder from '@/components/strategy/ai-builder';
import FormBuilder from '@/components/strategy/form-builder';
import NodeEditor from '@/components/strategy/node-editor';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { FormModeGraph, NodeModeGraph } from '@/types/models';

export type StrategyFormData = {
    name: string;
    description: string;
    mode: 'form' | 'node';
    graph: FormModeGraph | NodeModeGraph;
};

interface StrategyEditorFormProps {
    data: StrategyFormData;
    errors: Partial<Record<string, string>>;
    processing: boolean;
    initialFormGraph: FormModeGraph;
    initialNodeGraph: NodeModeGraph;
    submitLabel: string;
    submitHint: string;
    showAiBuilder?: boolean;
    onSubmit: (event: React.FormEvent) => void;
    onChange: (nextData: StrategyFormData) => void;
}

function isFormGraph(
    graph: FormModeGraph | NodeModeGraph,
): graph is FormModeGraph {
    return graph.mode === 'form';
}

function isNodeGraph(
    graph: FormModeGraph | NodeModeGraph,
): graph is NodeModeGraph {
    return graph.mode === 'node';
}

export default function StrategyEditorForm({
    data,
    errors,
    processing,
    initialFormGraph,
    initialNodeGraph,
    submitLabel,
    submitHint,
    showAiBuilder = false,
    onSubmit,
    onChange,
}: StrategyEditorFormProps) {
    function handleTabChange(tab: string): void {
        if (tab === 'form') {
            onChange({
                ...data,
                mode: 'form',
                graph: isFormGraph(data.graph) ? data.graph : initialFormGraph,
            });

            return;
        }

        onChange({
            ...data,
            mode: 'node',
            graph: isNodeGraph(data.graph) ? data.graph : initialNodeGraph,
        });
    }

    return (
        <form onSubmit={onSubmit} className="space-y-8">
            <div className="grid gap-6 sm:grid-cols-2">
                <div className="space-y-2">
                    <Label htmlFor="name">Name</Label>
                    <Input
                        id="name"
                        value={data.name}
                        onChange={(e) =>
                            onChange({ ...data, name: e.target.value })
                        }
                        placeholder="e.g. BTC Momentum Long"
                    />
                    <InputError message={errors.name} />
                </div>
                <div className="space-y-2">
                    <Label htmlFor="description">Description</Label>
                    <Input
                        id="description"
                        value={data.description}
                        onChange={(e) =>
                            onChange({ ...data, description: e.target.value })
                        }
                        placeholder="Describe what this strategy does..."
                    />
                </div>
            </div>

            {showAiBuilder && (
                <AiBuilder
                    onGenerated={(graph) =>
                        onChange({ ...data, mode: 'form', graph })
                    }
                />
            )}

            <Tabs value={data.mode} onValueChange={handleTabChange}>
                <TabsList>
                    <TabsTrigger value="form">Form Builder</TabsTrigger>
                    <TabsTrigger value="node">Node Editor</TabsTrigger>
                </TabsList>
                <TabsContent value="form" className="mt-6">
                    <FormBuilder
                        graph={
                            isFormGraph(data.graph)
                                ? data.graph
                                : initialFormGraph
                        }
                        onChange={(graph) => onChange({ ...data, graph })}
                    />
                </TabsContent>
                <TabsContent value="node" className="mt-6">
                    <NodeEditor
                        graph={
                            isNodeGraph(data.graph)
                                ? data.graph
                                : initialNodeGraph
                        }
                        onChange={(graph) => onChange({ ...data, graph })}
                    />
                </TabsContent>
            </Tabs>

            <InputError message={errors.graph} />

            <div className="sticky bottom-0 z-10 -mx-4 border-t bg-background/80 px-4 py-4 backdrop-blur-sm md:-mx-8 md:px-8">
                <div className="flex items-center gap-4">
                    <Button type="submit" size="lg" disabled={processing}>
                        {submitLabel}
                    </Button>
                    <p className="text-sm text-muted-foreground">
                        {submitHint}
                    </p>
                </div>
            </div>
        </form>
    );
}
