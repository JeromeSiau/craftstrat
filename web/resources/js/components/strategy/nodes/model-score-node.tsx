import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

const intervals = [
    { label: '1s', value: '1000' },
    { label: '2s', value: '2000' },
    { label: '5s', value: '5000' },
    { label: '15s', value: '15000' },
] as const;

type ModelScoreNodeData = {
    url: string;
    json_path: string;
    interval_ms: number;
    label: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function ModelScoreNode({ id, data }: NodeProps & { data: ModelScoreNodeData }) {
    return (
        <div className="w-52 rounded-md border-2 border-emerald-400 bg-emerald-50 p-2 shadow-sm dark:border-emerald-600 dark:bg-emerald-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-emerald-700 dark:text-emerald-400">
                Model Score
            </div>
            <div className="space-y-1">
                <Input
                    value={(data.label as string) ?? ''}
                    onChange={(e) => data.onUpdate(id, { ...data, label: e.target.value })}
                    placeholder="Label"
                    className="h-7 text-xs"
                />
                <Input
                    value={(data.url as string) ?? ''}
                    onChange={(e) => data.onUpdate(id, { ...data, url: e.target.value })}
                    placeholder="https://ml.example.com/predict"
                    className="h-7 text-xs"
                />
                <Input
                    value={(data.json_path as string) ?? ''}
                    onChange={(e) => data.onUpdate(id, { ...data, json_path: e.target.value })}
                    placeholder="proba_up"
                    className="h-7 text-xs"
                />
                <Select
                    value={String(data.interval_ms ?? 2000)}
                    onValueChange={(value) => data.onUpdate(id, { ...data, interval_ms: Number(value) })}
                >
                    <SelectTrigger className="h-7 text-xs">
                        <SelectValue placeholder="Interval" />
                    </SelectTrigger>
                    <SelectContent>
                        {intervals.map((interval) => (
                            <SelectItem key={interval.value} value={interval.value}>
                                {interval.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <Handle type="source" position={Position.Right} className="!bg-emerald-400" />
        </div>
    );
}
