import { uid } from '@/lib/formatters';
import type { FormModeGraph, NodeModeGraph } from '@/types/models';

export function createDefaultFormGraph(): FormModeGraph {
    return {
        mode: 'form',
        conditions: [
            {
                id: uid(),
                type: 'AND',
                rules: [
                    {
                        id: uid(),
                        indicator: 'abs_move_pct',
                        operator: '>',
                        value: 3.0,
                    },
                ],
            },
        ],
        action: {
            signal: 'buy',
            outcome: 'UP',
            size_mode: 'fixed',
            size_usdc: 50,
            order_type: 'market',
            limit_price: null,
        },
        risk: {
            stoploss_pct: null,
            take_profit_pct: null,
            max_position_usdc: 200,
            max_trades_per_slot: 1,
            daily_loss_limit_usdc: null,
            cooldown_seconds: null,
            prevent_duplicates: false,
        },
    };
}

export function createDefaultNodeGraph(): NodeModeGraph {
    return {
        mode: 'node',
        nodes: [
            {
                id: 'n1',
                type: 'input',
                data: { field: 'abs_move_pct' },
                position: { x: 50, y: 100 },
            },
            {
                id: 'n2',
                type: 'comparator',
                data: { operator: '>', value: 3.0 },
                position: { x: 300, y: 100 },
            },
            {
                id: 'n3',
                type: 'action',
                data: {
                    signal: 'buy',
                    outcome: 'UP',
                    size_usdc: 50,
                    order_type: 'market',
                    limit_price: null,
                },
                position: { x: 550, y: 100 },
            },
        ],
        edges: [
            { source: 'n1', target: 'n2' },
            { source: 'n2', target: 'n3' },
        ],
    };
}
