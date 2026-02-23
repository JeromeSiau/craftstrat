import { Handle, Position } from '@xyflow/react';

export default function NotNode() {
    return (
        <div className="rounded-md border-2 border-blue-400 bg-blue-50 p-2 shadow-sm dark:border-blue-600 dark:bg-blue-950">
            <div className="text-center text-[10px] font-semibold uppercase tracking-wide text-blue-600 dark:text-blue-400">
                NOT
            </div>
            <Handle type="target" position={Position.Left} className="!bg-blue-400" />
            <Handle type="source" position={Position.Right} className="!bg-blue-400" />
        </div>
    );
}
