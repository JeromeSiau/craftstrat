import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Textarea } from '@/components/ui/textarea';
import type { FormModeGraph } from '@/types/models';

interface AiBuilderProps {
    onGenerated: (graph: FormModeGraph) => void;
}

export default function AiBuilder({ onGenerated }: AiBuilderProps) {
    const [description, setDescription] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    async function handleGenerate() {
        setLoading(true);
        setError(null);

        try {
            const response = await fetch('/strategies/generate', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json',
                    'X-XSRF-TOKEN': decodeURIComponent(
                        document.cookie
                            .split('; ')
                            .find((c) => c.startsWith('XSRF-TOKEN='))
                            ?.split('=')[1] ?? '',
                    ),
                },
                body: JSON.stringify({ description }),
            });

            const data = await response.json();

            if (!response.ok) {
                setError(data.error ?? data.message ?? 'Generation failed.');
                return;
            }

            onGenerated(data.graph as FormModeGraph);
            setDescription('');
        } catch {
            setError('Network error. Please try again.');
        } finally {
            setLoading(false);
        }
    }

    return (
        <Card className="border-dashed">
            <CardContent className="pt-6">
                <div className="space-y-3">
                    <div className="flex items-center gap-2">
                        <span className="text-sm font-medium">AI Strategy Builder</span>
                        <span className="rounded bg-muted px-1.5 py-0.5 text-[10px] font-medium uppercase text-muted-foreground">
                            Beta
                        </span>
                    </div>
                    <Textarea
                        placeholder="Describe your strategy in plain English, e.g. &quot;Buy UP when price drops more than 5% and the spread is tight&quot;"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        rows={2}
                        className="resize-none text-sm"
                    />
                    {error && <p className="text-sm text-destructive">{error}</p>}
                    <Button
                        type="button"
                        variant="secondary"
                        size="sm"
                        disabled={loading || description.length < 10}
                        onClick={handleGenerate}
                    >
                        {loading ? 'Generating...' : 'Generate Strategy'}
                    </Button>
                </div>
            </CardContent>
        </Card>
    );
}
