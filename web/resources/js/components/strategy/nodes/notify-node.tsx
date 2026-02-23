import { Handle, Position } from '@xyflow/react';
import type { NodeProps } from '@xyflow/react';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

type NotifyNodeData = {
    channel: string;
    message: string;
    onUpdate: (id: string, data: Record<string, unknown>) => void;
    [key: string]: unknown;
};

export default function NotifyNode({ id, data }: NodeProps & { data: NotifyNodeData }) {
    return (
        <div className="rounded-md border-2 border-yellow-400 bg-yellow-50 p-2 shadow-sm dark:border-yellow-600 dark:bg-yellow-950">
            <div className="mb-1 text-[10px] font-semibold uppercase tracking-wide text-yellow-600 dark:text-yellow-400">
                Notify
            </div>
            <div className="space-y-1">
                <Select
                    value={data.channel as string}
                    onValueChange={(value) => data.onUpdate(id, { ...data, channel: value })}
                >
                    <SelectTrigger className="h-7 text-xs">
                        <SelectValue placeholder="Channel" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="database">In-App</SelectItem>
                        <SelectItem value="mail">Email</SelectItem>
                    </SelectContent>
                </Select>
                <Input
                    type="text"
                    value={data.message as string}
                    onChange={(e) => data.onUpdate(id, { ...data, message: e.target.value })}
                    placeholder="Alert message..."
                    className="h-7 text-xs"
                />
            </div>
            <Handle type="target" position={Position.Left} className="!bg-yellow-400" />
        </div>
    );
}
