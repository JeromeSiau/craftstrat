import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Input } from '@/components/ui/input';

type KellyNodeData = {
    fraction: number;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function KellyNode({ id, data }: NodeProps & { data: KellyNodeData }) {
    return (
        <div className="rounded-md border-2 border-emerald-400 bg-emerald-50 p-2 shadow-sm dark:border-emerald-600 dark:bg-emerald-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-emerald-600 dark:text-emerald-400">
                Kelly Sizer
            </div>
            <Input
                type="number"
                step="0.1"
                min="0"
                max="1"
                value={data.fraction as number}
                onChange={(e) => data.onUpdate(id, { ...data, fraction: Number(e.target.value) })}
                placeholder="Fraction"
                className="h-7 text-xs"
            />
            <div className="mt-1 flex justify-between text-[9px] text-muted-foreground">
                <span>prob</span>
                <span>price</span>
            </div>
            <Handle type="target" id="prob" position={Position.Left} className="!bg-emerald-400" style={{ top: '30%' }} />
            <Handle type="target" id="price" position={Position.Left} className="!bg-emerald-400" style={{ top: '70%' }} />
            <Handle type="source" position={Position.Right} className="!bg-emerald-400" />
        </div>
    );
}
