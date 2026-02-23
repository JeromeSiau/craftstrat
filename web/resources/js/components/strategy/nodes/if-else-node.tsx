import { Handle, Position } from '@xyflow/react';

export default function IfElseNode() {
    return (
        <div className="rounded-md border-2 border-amber-400 bg-amber-50 p-2 shadow-sm dark:border-amber-600 dark:bg-amber-950">
            <div className="mb-1 text-center text-[10px] font-semibold uppercase tracking-wide text-amber-600 dark:text-amber-400">
                IF / ELSE
            </div>
            <div className="flex items-center justify-between gap-3 text-[10px]">
                <span className="text-green-600 dark:text-green-400">T</span>
                <span className="text-red-600 dark:text-red-400">F</span>
            </div>
            <Handle type="target" position={Position.Left} className="!bg-amber-400" />
            <Handle type="source" id="true" position={Position.Right} className="!bg-green-500" style={{ top: '30%' }} />
            <Handle type="source" id="false" position={Position.Right} className="!bg-red-500" style={{ top: '70%' }} />
        </div>
    );
}
