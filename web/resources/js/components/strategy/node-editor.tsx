import { useCallback, useMemo, useRef } from 'react';
import {
    ReactFlow,
    Background,
    Controls,
    useNodesState,
    useEdgesState,
    addEdge,
    type Node,
    type Edge,
    type OnConnect,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Button } from '@/components/ui/button';
import InputNode from '@/components/strategy/nodes/input-node';
import IndicatorNode from '@/components/strategy/nodes/indicator-node';
import ComparatorNode from '@/components/strategy/nodes/comparator-node';
import LogicNode from '@/components/strategy/nodes/logic-node';
import ActionNode from '@/components/strategy/nodes/action-node';
import type { NodeModeGraph } from '@/types/models';

interface NodeEditorProps {
    graph: NodeModeGraph;
    onChange: (graph: NodeModeGraph) => void;
}

const nodeDefaults: Record<string, Record<string, unknown>> = {
    input: { field: 'abs_move_pct' },
    indicator: { fn: 'EMA', period: 20, field: 'mid_up' },
    comparator: { operator: '>', value: 0 },
    logic: { operator: 'AND' },
    action: { signal: 'buy', outcome: 'UP', size_usdc: 50 },
};

function toFlowNodes(
    graphNodes: NodeModeGraph['nodes'],
    onUpdate: (id: string, data: Record<string, unknown>) => void,
): Node[] {
    return graphNodes.map((node) => ({
        id: node.id,
        type: node.type,
        position: node.position ?? { x: 0, y: 0 },
        data: { ...node.data, onUpdate },
    }));
}

function toFlowEdges(graphEdges: NodeModeGraph['edges']): Edge[] {
    return graphEdges.map((edge) => ({
        id: `${edge.source}-${edge.target}`,
        source: edge.source,
        target: edge.target,
    }));
}

export default function NodeEditor({ graph, onChange }: NodeEditorProps) {
    const counterRef = useRef(
        graph.nodes.reduce((max, n) => {
            const num = parseInt(n.id.replace('n', ''), 10);
            return isNaN(num) ? max : Math.max(max, num);
        }, 0),
    );

    const handleNodeDataUpdate = useCallback(
        (nodeId: string, newData: Record<string, unknown>) => {
            setNodes((prevNodes) =>
                prevNodes.map((node) =>
                    node.id === nodeId ? { ...node, data: { ...newData, onUpdate: handleNodeDataUpdate } } : node,
                ),
            );
        },
        [],
    );

    const [nodes, setNodes, onNodesChange] = useNodesState(toFlowNodes(graph.nodes, handleNodeDataUpdate));
    const [edges, setEdges, onEdgesChange] = useEdgesState(toFlowEdges(graph.edges));

    const nodeTypes = useMemo(
        () => ({
            input: InputNode,
            indicator: IndicatorNode,
            comparator: ComparatorNode,
            logic: LogicNode,
            action: ActionNode,
        }),
        [],
    );

    const onConnect: OnConnect = useCallback(
        (connection) => {
            setEdges((eds) => addEdge(connection, eds));
        },
        [setEdges],
    );

    function addNode(type: string): void {
        counterRef.current += 1;
        const id = `n${counterRef.current}`;
        const newNode: Node = {
            id,
            type,
            position: { x: 100 + Math.random() * 200, y: 50 + Math.random() * 300 },
            data: { ...nodeDefaults[type], onUpdate: handleNodeDataUpdate },
        };
        setNodes((prev) => [...prev, newNode]);
    }

    function handleSave(): void {
        const graphNodes = nodes.map((node) => {
            const { onUpdate, ...rest } = node.data as Record<string, unknown>;
            return {
                id: node.id,
                type: node.type as NodeModeGraph['nodes'][number]['type'],
                data: rest,
                position: node.position,
            };
        });
        const graphEdges = edges.map((edge) => ({
            source: edge.source,
            target: edge.target,
        }));
        onChange({ mode: 'node', nodes: graphNodes, edges: graphEdges });
    }

    return (
        <div className="space-y-3">
            <div className="flex flex-wrap items-center gap-2">
                <span className="text-xs font-medium text-muted-foreground">Add:</span>
                <Button type="button" variant="outline" size="sm" className="h-7 text-xs" onClick={() => addNode('input')}>
                    + Input
                </Button>
                <Button type="button" variant="outline" size="sm" className="h-7 text-xs" onClick={() => addNode('indicator')}>
                    + Indicator
                </Button>
                <Button type="button" variant="outline" size="sm" className="h-7 text-xs" onClick={() => addNode('comparator')}>
                    + Compare
                </Button>
                <Button type="button" variant="outline" size="sm" className="h-7 text-xs" onClick={() => addNode('logic')}>
                    + Logic
                </Button>
                <Button type="button" variant="outline" size="sm" className="h-7 text-xs" onClick={() => addNode('action')}>
                    + Action
                </Button>
                <div className="flex-1" />
                <Button type="button" size="sm" className="h-7 text-xs" onClick={handleSave}>
                    Save Graph
                </Button>
            </div>
            <div className="h-[500px] rounded-md border">
                <ReactFlow
                    nodes={nodes}
                    edges={edges}
                    onNodesChange={onNodesChange}
                    onEdgesChange={onEdgesChange}
                    onConnect={onConnect}
                    nodeTypes={nodeTypes}
                    fitView
                >
                    <Background />
                    <Controls />
                </ReactFlow>
            </div>
        </div>
    );
}
