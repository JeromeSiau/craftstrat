import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

const intervals = [
    { label: '30s', value: '30' },
    { label: '1m', value: '60' },
    { label: '5m', value: '300' },
    { label: '15m', value: '900' },
] as const;

type ApiFetchNodeData = {
    url: string;
    json_path: string;
    interval_secs: number;
    label: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function ApiFetchNode({ id, data }: NodeProps & { data: ApiFetchNodeData }) {
    return (
        <div className="w-48 rounded-md border-2 border-cyan-400 bg-cyan-50 p-2 shadow-sm dark:border-cyan-600 dark:bg-cyan-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-cyan-600 dark:text-cyan-400">
                API Fetch
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
                    placeholder="https://api.example.com/data"
                    className="h-7 text-xs"
                />
                <Input
                    value={(data.json_path as string) ?? ''}
                    onChange={(e) => data.onUpdate(id, { ...data, json_path: e.target.value })}
                    placeholder="$.main.temp"
                    className="h-7 text-xs"
                />
                <Select
                    value={String(data.interval_secs ?? 60)}
                    onValueChange={(value) => data.onUpdate(id, { ...data, interval_secs: Number(value) })}
                >
                    <SelectTrigger className="h-7 text-xs">
                        <SelectValue placeholder="Interval" />
                    </SelectTrigger>
                    <SelectContent>
                        {intervals.map((i) => (
                            <SelectItem key={i.value} value={i.value}>
                                {i.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
            </div>
            <Handle type="source" position={Position.Right} className="!bg-cyan-400" />
        </div>
    );
}
