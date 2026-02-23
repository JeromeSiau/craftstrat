import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

type EvCalculatorNodeData = {
    mode: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function EvCalculatorNode({ id, data }: NodeProps & { data: EvCalculatorNodeData }) {
    return (
        <div className="rounded-md border-2 border-orange-400 bg-orange-50 p-2 shadow-sm dark:border-orange-600 dark:bg-orange-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-orange-600 dark:text-orange-400">
                EV Calc
            </div>
            <Select
                value={data.mode as string}
                onValueChange={(value) => data.onUpdate(id, { ...data, mode: value })}
            >
                <SelectTrigger className="h-7 text-xs">
                    <SelectValue placeholder="Mode" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="simple">Simple</SelectItem>
                    <SelectItem value="custom">Custom</SelectItem>
                </SelectContent>
            </Select>
            <div className="mt-1 flex justify-between text-[9px] text-muted-foreground">
                <span>price</span>
                <span>prob</span>
            </div>
            <Handle type="target" id="price" position={Position.Left} className="!bg-orange-400" style={{ top: '30%' }} />
            <Handle type="target" id="prob" position={Position.Left} className="!bg-orange-400" style={{ top: '70%' }} />
            <Handle type="source" position={Position.Right} className="!bg-orange-400" />
        </div>
    );
}
