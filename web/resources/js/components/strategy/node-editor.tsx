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
import { useCallback, useEffect, useMemo, useRef } from 'react';
import '@xyflow/react/dist/style.css';
import ActionNode from '@/components/strategy/nodes/action-node';
import ApiFetchNode from '@/components/strategy/nodes/api-fetch-node';
import CancelNode from '@/components/strategy/nodes/cancel-node';
import ComparatorNode from '@/components/strategy/nodes/comparator-node';
import EvCalculatorNode from '@/components/strategy/nodes/ev-calculator-node';
import IfElseNode from '@/components/strategy/nodes/if-else-node';
import IndicatorNode from '@/components/strategy/nodes/indicator-node';
import InputNode from '@/components/strategy/nodes/input-node';
import KellyNode from '@/components/strategy/nodes/kelly-node';
import LogicNode from '@/components/strategy/nodes/logic-node';
import MathNode from '@/components/strategy/nodes/math-node';
import ModelScoreNode from '@/components/strategy/nodes/model-score-node';
import NotNode from '@/components/strategy/nodes/not-node';
import NotifyNode from '@/components/strategy/nodes/notify-node';
import { Button } from '@/components/ui/button';
import type { GraphValue, NodeModeGraph } from '@/types/models';

interface NodeEditorProps {
    graph: NodeModeGraph;
    onChange: (graph: NodeModeGraph) => void;
}

const nodeDefaults: Record<string, Record<string, GraphValue>> = {
    input: { field: 'abs_move_pct' },
    indicator: { fn: 'EMA', period: 20, field: 'mid_up' },
    comparator: { operator: '>', value: 0 },
    logic: { operator: 'AND' },
    action: { signal: 'buy', outcome: 'UP', size_usdc: 50 },
    not: {},
    if_else: {},
    math: { operation: '+' },
    ev_calculator: { mode: 'simple' },
    kelly: { fraction: 0.5 },
    cancel: { outcome: 'UP' },
    notify: { channel: 'database', message: 'Strategy alert' },
    api_fetch: {
        url: '',
        json_path: '',
        interval_secs: 60,
        label: 'API Value',
    },
    model_score: {
        url: '',
        json_path: 'proba_up',
        interval_ms: 2000,
        label: 'Model Score',
    },
};

function toFlowNodes(
    graphNodes: NodeModeGraph['nodes'],
    onUpdate: (id: string, data: Record<string, GraphValue>) => void,
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
        id: `${edge.source}-${edge.target}-${edge.sourceHandle ?? ''}-${edge.targetHandle ?? ''}`,
        source: edge.source,
        target: edge.target,
        sourceHandle: edge.sourceHandle ?? undefined,
        targetHandle: edge.targetHandle ?? undefined,
    }));
}

export default function NodeEditor({ graph, onChange }: NodeEditorProps) {
    const counterRef = useRef(
        graph.nodes.reduce((max, n) => {
            const num = parseInt(n.id.replace('n', ''), 10);
            return isNaN(num) ? max : Math.max(max, num);
        }, 0),
    );

    const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
    const handleNodeDataUpdateRef = useRef<
        (nodeId: string, newData: Record<string, GraphValue>) => void
    >(() => undefined);
    const proxyNodeDataUpdate = useCallback(
        (nodeId: string, newData: Record<string, GraphValue>) => {
            handleNodeDataUpdateRef.current(nodeId, newData);
        },
        [],
    );

    const handleNodeDataUpdate = useCallback(
        (nodeId: string, newData: Record<string, GraphValue>) => {
            setNodes((prevNodes) =>
                prevNodes.map((node) =>
                    node.id === nodeId
                        ? {
                              ...node,
                              data: {
                                  ...newData,
                                  onUpdate: proxyNodeDataUpdate,
                              },
                          }
                        : node,
                ),
            );
        },
        [proxyNodeDataUpdate, setNodes],
    );

    useEffect(() => {
        handleNodeDataUpdateRef.current = handleNodeDataUpdate;
    }, [handleNodeDataUpdate]);

    useEffect(() => {
        setNodes(toFlowNodes(graph.nodes, proxyNodeDataUpdate));
        setEdges(toFlowEdges(graph.edges));
    }, [graph.edges, graph.nodes, proxyNodeDataUpdate, setEdges, setNodes]);

    const nodeTypes = useMemo(
        () => ({
            input: InputNode,
            indicator: IndicatorNode,
            comparator: ComparatorNode,
            logic: LogicNode,
            action: ActionNode,
            not: NotNode,
            if_else: IfElseNode,
            math: MathNode,
            ev_calculator: EvCalculatorNode,
            kelly: KellyNode,
            cancel: CancelNode,
            notify: NotifyNode,
            api_fetch: ApiFetchNode,
            model_score: ModelScoreNode,
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
            position: {
                x: 100 + Math.random() * 200,
                y: 50 + Math.random() * 300,
            },
            data: { ...nodeDefaults[type], onUpdate: handleNodeDataUpdate },
        };
        setNodes((prev) => [...prev, newNode]);
    }

    function handleSave(): void {
        const graphNodes = nodes.map((node) => {
            const rest = { ...(node.data as Record<string, GraphValue>) };
            delete rest.onUpdate;
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
            sourceHandle: edge.sourceHandle ?? null,
            targetHandle: edge.targetHandle ?? null,
        }));
        onChange({ mode: 'node', nodes: graphNodes, edges: graphEdges });
    }

    return (
        <div className="space-y-3">
            <div className="flex flex-wrap items-center gap-2">
                <span className="text-xs font-medium text-muted-foreground">
                    Add:
                </span>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('input')}
                >
                    + Input
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('indicator')}
                >
                    + Indicator
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('comparator')}
                >
                    + Compare
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('logic')}
                >
                    + Logic
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('action')}
                >
                    + Action
                </Button>
                <span className="text-muted-foreground">|</span>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('not')}
                >
                    + NOT
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('if_else')}
                >
                    + IF/ELSE
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('math')}
                >
                    + Math
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('ev_calculator')}
                >
                    + EV Calc
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('kelly')}
                >
                    + Kelly
                </Button>
                <span className="text-muted-foreground">|</span>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('cancel')}
                >
                    + Cancel
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('notify')}
                >
                    + Notify
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('api_fetch')}
                >
                    + API Fetch
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => addNode('model_score')}
                >
                    + Model Score
                </Button>
                <div className="flex-1" />
                <Button
                    type="button"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={handleSave}
                >
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
