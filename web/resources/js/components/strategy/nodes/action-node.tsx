import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';

type ActionNodeData = {
    signal: string;
    outcome: string;
    size_usdc: number;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function ActionNode({ id, data }: NodeProps & { data: ActionNodeData }) {
    return (
        <div className="rounded-md border-2 border-green-400 bg-green-50 p-2 shadow-sm dark:border-green-600 dark:bg-green-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-green-600 dark:text-green-400">
                Action
            </div>
            <div className="space-y-1">
                <Select
                    value={data.signal as string}
                    onValueChange={(value) => data.onUpdate(id, { ...data, signal: value })}
                >
                    <SelectTrigger className="h-7 text-xs">
                        <SelectValue placeholder="Signal" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="buy">Buy</SelectItem>
                        <SelectItem value="sell">Sell</SelectItem>
                    </SelectContent>
                </Select>
                <Select
                    value={data.outcome as string}
                    onValueChange={(value) => data.onUpdate(id, { ...data, outcome: value })}
                >
                    <SelectTrigger className="h-7 text-xs">
                        <SelectValue placeholder="Outcome" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="UP">UP</SelectItem>
                        <SelectItem value="DOWN">DOWN</SelectItem>
                    </SelectContent>
                </Select>
                <Input
                    type="number"
                    value={data.size_usdc as number}
                    onChange={(e) => data.onUpdate(id, { ...data, size_usdc: Number(e.target.value) })}
                    placeholder="Size (USDC)"
                    className="h-7 text-xs"
                />
            </div>
            <Handle type="target" position={Position.Left} className="!bg-green-400" />
        </div>
    );
}
