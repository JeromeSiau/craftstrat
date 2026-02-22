# Phase 8 — Frontend Inertia/React Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build all Inertia/React frontend pages with real data, strategy builder (form + node editor), backtest charts, wallet management, and billing upgrade flow.

**Architecture:** Existing Laravel controllers pass Inertia props to React pages. Sidebar navigation links all sections. Strategy builder uses a form-based mode (SI/ET/ALORS) and a React Flow node editor mode. Charts use Recharts. All styling via Tailwind CSS v4 + shadcn/ui.

**Tech Stack:** React 19, Inertia.js v2, Tailwind CSS v4, shadcn/ui, Recharts, @xyflow/react (React Flow v12), Wayfinder route helpers, Pest 4 for backend tests.

**Current state:** Sidebar has only "Dashboard". Dashboard is placeholder. Strategy create page accepts name/description only (no builder). Backtest show page has metric cards but no charts. Wallet page lacks strategy assignment. Billing page lacks upgrade buttons.

---

## Task 1: Sidebar Navigation + Dependencies

**Files:**
- Modify: `web/resources/js/components/app-sidebar.tsx`
- Modify: `web/package.json` (via npm install)

**Step 1: Add navigation items to sidebar**

Update `web/resources/js/components/app-sidebar.tsx` to add all nav links:

```tsx
import { Link } from '@inertiajs/react';
import { BookOpen, CreditCard, Folder, LayoutGrid, LineChart, Target, Wallet } from 'lucide-react';
import { NavFooter } from '@/components/nav-footer';
import { NavMain } from '@/components/nav-main';
import { NavUser } from '@/components/nav-user';
import {
    Sidebar, SidebarContent, SidebarFooter, SidebarHeader,
    SidebarMenu, SidebarMenuButton, SidebarMenuItem,
} from '@/components/ui/sidebar';
import type { NavItem } from '@/types';
import AppLogo from './app-logo';
import { dashboard } from '@/routes';
import { index as strategiesIndex } from '@/actions/App/Http/Controllers/StrategyController';
import { index as walletsIndex } from '@/actions/App/Http/Controllers/WalletController';
import { index as backtestsIndex } from '@/actions/App/Http/Controllers/BacktestController';
import { index as billingIndex } from '@/actions/App/Http/Controllers/BillingController';

const mainNavItems: NavItem[] = [
    { title: 'Dashboard', href: dashboard(), icon: LayoutGrid },
    { title: 'Strategies', href: strategiesIndex.url(), icon: Target },
    { title: 'Wallets', href: walletsIndex.url(), icon: Wallet },
    { title: 'Backtests', href: backtestsIndex.url(), icon: LineChart },
    { title: 'Billing', href: billingIndex.url(), icon: CreditCard },
];

const footerNavItems: NavItem[] = [
    { title: 'Documentation', href: 'https://docs.craftstrat.com', icon: BookOpen },
];
```

Keep the rest of the component the same (SidebarHeader, SidebarContent with NavMain, SidebarFooter with NavFooter + NavUser).

**Step 2: Verify sidebar renders correctly**

Run: `cd web && npm run build`
Expected: Build succeeds without errors.

**Step 3: Install Recharts for charts**

Run: `cd web && npm install recharts`

**Step 4: Install React Flow for node editor**

Run: `cd web && npm install @xyflow/react`

**Step 5: Add missing shadcn/ui components (tabs, textarea, table, scroll-area)**

Run:
```bash
cd web && npx shadcn@latest add tabs textarea table scroll-area --yes
```

If shadcn CLI is not configured, create these manually following shadcn/ui patterns from existing components.

**Step 6: Verify build with new dependencies**

Run: `cd web && npm run build`
Expected: Build succeeds.

**Step 7: Run existing tests**

Run: `cd web && php artisan test --compact`
Expected: All tests pass.

**Step 8: Commit**

```bash
git add web/resources/js/components/app-sidebar.tsx web/package.json web/package-lock.json web/resources/js/components/ui/
git commit -m "feat(nav): add sidebar navigation links and install chart/flow dependencies"
```

---

## Task 2: Dashboard — Backend Data

**Files:**
- Modify: `web/app/Http/Controllers/DashboardController.php`
- Create: `web/tests/Feature/DashboardControllerTest.php` (if not exists)
- Modify: `web/resources/js/types/models.ts`

**Step 1: Write failing test for dashboard data**

Create test via `cd web && php artisan make:test DashboardControllerTest --pest` (if not exists).

```php
<?php

use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;

it('returns dashboard stats as Inertia props', function () {
    $user = User::factory()->create();

    Strategy::factory()->count(2)->for($user)->create(['is_active' => true]);
    Strategy::factory()->for($user)->create(['is_active' => false]);
    Wallet::factory()->count(3)->for($user)->create();

    $this->actingAs($user)
        ->get('/dashboard')
        ->assertInertia(fn ($page) => $page
            ->component('dashboard')
            ->has('stats')
            ->where('stats.active_strategies', 2)
            ->where('stats.total_wallets', 3)
            ->where('stats.total_strategies', 3)
        );
});
```

**Step 2: Run test to verify it fails**

Run: `cd web && php artisan test --compact --filter=DashboardControllerTest`
Expected: FAIL — `stats` prop not present.

**Step 3: Update DashboardController with stats**

```php
<?php

namespace App\Http\Controllers;

use Inertia\Inertia;
use Inertia\Response;

class DashboardController extends Controller
{
    public function index(): Response
    {
        $user = auth()->user();

        return Inertia::render('dashboard', [
            'stats' => [
                'active_strategies' => $user->strategies()->where('is_active', true)->count(),
                'total_strategies' => $user->strategies()->count(),
                'total_wallets' => $user->wallets()->count(),
                'total_pnl_usdc' => $user->wallets()
                    ->join('trades', 'wallets.id', '=', 'trades.wallet_id')
                    ->where('trades.status', 'filled')
                    ->sum('trades.size_usdc'),
                'running_assignments' => $user->wallets()
                    ->join('wallet_strategies', 'wallets.id', '=', 'wallet_strategies.wallet_id')
                    ->where('wallet_strategies.is_running', true)
                    ->count(),
            ],
            'recentStrategies' => $user->strategies()
                ->withCount('wallets')
                ->latest()
                ->limit(5)
                ->get(),
        ]);
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd web && php artisan test --compact --filter=DashboardControllerTest`
Expected: PASS.

**Step 5: Add DashboardStats type to TypeScript**

Add to `web/resources/js/types/models.ts`:

```typescript
export interface DashboardStats {
    active_strategies: number;
    total_strategies: number;
    total_wallets: number;
    total_pnl_usdc: string;
    running_assignments: number;
}
```

**Step 6: Run Pint**

Run: `cd web && vendor/bin/pint --dirty --format agent`

**Step 7: Commit**

```bash
git add web/app/Http/Controllers/DashboardController.php web/tests/Feature/DashboardControllerTest.php web/resources/js/types/models.ts
git commit -m "feat(dashboard): add stats and recent strategies to dashboard controller"
```

---

## Task 3: Dashboard — Frontend

**Files:**
- Modify: `web/resources/js/pages/dashboard.tsx`

**Step 1: Replace placeholder dashboard with real data**

Rewrite `web/resources/js/pages/dashboard.tsx`:

```tsx
import { Head, Link } from '@inertiajs/react';
import { Activity, LineChart, Target, Wallet } from 'lucide-react';
import AppLayout from '@/layouts/app-layout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import type { BreadcrumbItem } from '@/types';
import type { DashboardStats, Strategy } from '@/types/models';
import { dashboard } from '@/routes';
import { show as strategyShow } from '@/actions/App/Http/Controllers/StrategyController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Dashboard', href: dashboard().url },
];

interface Props {
    stats: DashboardStats;
    recentStrategies: Strategy[];
}

export default function Dashboard({ stats, recentStrategies }: Props) {
    const cards = [
        { label: 'Active Strategies', value: stats.active_strategies, icon: Target },
        { label: 'Total Wallets', value: stats.total_wallets, icon: Wallet },
        { label: 'Running Assignments', value: stats.running_assignments, icon: Activity },
        {
            label: 'Total PnL',
            value: `$${parseFloat(stats.total_pnl_usdc || '0').toFixed(2)}`,
            icon: LineChart,
        },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Dashboard" />
            <div className="flex flex-1 flex-col gap-6 p-6">
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    {cards.map((card) => (
                        <Card key={card.label}>
                            <CardHeader className="flex flex-row items-center justify-between pb-2">
                                <CardTitle className="text-sm font-medium text-muted-foreground">
                                    {card.label}
                                </CardTitle>
                                <card.icon className="size-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent>
                                <p className="text-2xl font-bold">{card.value}</p>
                            </CardContent>
                        </Card>
                    ))}
                </div>

                <Card>
                    <CardHeader>
                        <CardTitle>Recent Strategies</CardTitle>
                    </CardHeader>
                    <CardContent>
                        {recentStrategies.length === 0 ? (
                            <p className="text-sm text-muted-foreground">No strategies yet.</p>
                        ) : (
                            <div className="space-y-3">
                                {recentStrategies.map((strategy) => (
                                    <Link
                                        key={strategy.id}
                                        href={strategyShow.url(strategy.id)}
                                        className="flex items-center justify-between rounded-lg border p-3 transition hover:bg-accent"
                                    >
                                        <div>
                                            <p className="font-medium">{strategy.name}</p>
                                            <p className="text-sm text-muted-foreground">
                                                {strategy.mode} mode · {strategy.wallets_count ?? 0} wallet(s)
                                            </p>
                                        </div>
                                        <span
                                            className={`rounded-full px-2 py-1 text-xs font-medium ${
                                                strategy.is_active
                                                    ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                                                    : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
                                            }`}
                                        >
                                            {strategy.is_active ? 'Active' : 'Inactive'}
                                        </span>
                                    </Link>
                                ))}
                            </div>
                        )}
                    </CardContent>
                </Card>
            </div>
        </AppLayout>
    );
}
```

**Step 2: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds.

**Step 3: Run tests**

Run: `cd web && php artisan test --compact --filter=DashboardControllerTest`
Expected: PASS.

**Step 4: Commit**

```bash
git add web/resources/js/pages/dashboard.tsx
git commit -m "feat(dashboard): replace placeholder with stats cards and recent strategies"
```

---

## Task 4: Strategy Form Builder — Components

This is the core of the strategy builder. Creates reusable components for the "SI / ET / ALORS" form mode.

**Files:**
- Create: `web/resources/js/components/strategy/indicator-options.ts`
- Create: `web/resources/js/components/strategy/rule-row.tsx`
- Create: `web/resources/js/components/strategy/condition-group.tsx`
- Create: `web/resources/js/components/strategy/action-config.tsx`
- Create: `web/resources/js/components/strategy/risk-config.tsx`
- Create: `web/resources/js/components/strategy/form-builder.tsx`
- Modify: `web/resources/js/types/models.ts`

**Step 1: Add strategy graph types**

Add to `web/resources/js/types/models.ts`:

```typescript
// Strategy graph types for form mode
export interface StrategyRule {
    indicator: string;
    operator: string;
    value: number | [number, number];
}

export interface ConditionGroup {
    type: 'AND' | 'OR';
    rules: StrategyRule[];
}

export interface StrategyAction {
    signal: 'buy' | 'sell';
    outcome: 'UP' | 'DOWN';
    size_mode: 'fixed' | 'proportional';
    size_usdc: number;
    order_type: 'market' | 'limit';
}

export interface StrategyRisk {
    stoploss_pct: number;
    take_profit_pct: number;
    max_position_usdc: number;
    max_trades_per_slot: number;
}

export interface FormModeGraph {
    mode: 'form';
    conditions: ConditionGroup[];
    action: StrategyAction;
    risk: StrategyRisk;
}
```

**Step 2: Create indicator options data file**

Create `web/resources/js/components/strategy/indicator-options.ts`:

```typescript
export const indicators = [
    { value: 'abs_move_pct', label: 'Abs Move %', category: 'Price' },
    { value: 'dir_move_pct', label: 'Dir Move %', category: 'Price' },
    { value: 'spread_up', label: 'Spread UP', category: 'Spread' },
    { value: 'spread_down', label: 'Spread DOWN', category: 'Spread' },
    { value: 'size_ratio_up', label: 'Size Ratio UP', category: 'Order Book' },
    { value: 'size_ratio_down', label: 'Size Ratio DOWN', category: 'Order Book' },
    { value: 'pct_into_slot', label: '% Into Slot', category: 'Time' },
    { value: 'minutes_into_slot', label: 'Minutes Into Slot', category: 'Time' },
    { value: 'mid_up', label: 'Mid UP', category: 'Price' },
    { value: 'mid_down', label: 'Mid DOWN', category: 'Price' },
    { value: 'bid_up', label: 'Bid UP', category: 'Order Book' },
    { value: 'ask_up', label: 'Ask UP', category: 'Order Book' },
    { value: 'bid_down', label: 'Bid DOWN', category: 'Order Book' },
    { value: 'ask_down', label: 'Ask DOWN', category: 'Order Book' },
    { value: 'ref_price', label: 'Reference Price', category: 'Price' },
    { value: 'hour_utc', label: 'Hour (UTC)', category: 'Time' },
    { value: 'day_of_week', label: 'Day of Week', category: 'Time' },
    { value: 'market_volume_usd', label: 'Volume (USD)', category: 'Volume' },
] as const;

export const operators = [
    { value: '>', label: '>' },
    { value: '<', label: '<' },
    { value: '>=', label: '>=' },
    { value: '<=', label: '<=' },
    { value: '==', label: '=' },
    { value: '!=', label: '!=' },
    { value: 'between', label: 'Between' },
] as const;
```

**Step 3: Create RuleRow component**

Create `web/resources/js/components/strategy/rule-row.tsx`:

```tsx
import { Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { indicators, operators } from './indicator-options';
import type { StrategyRule } from '@/types/models';

interface RuleRowProps {
    rule: StrategyRule;
    onChange: (rule: StrategyRule) => void;
    onRemove: () => void;
}

export function RuleRow({ rule, onChange, onRemove }: RuleRowProps) {
    const isBetween = rule.operator === 'between';

    return (
        <div className="flex items-center gap-2">
            <Select
                value={rule.indicator}
                onValueChange={(v) => onChange({ ...rule, indicator: v })}
            >
                <SelectTrigger className="w-44">
                    <SelectValue placeholder="Indicator" />
                </SelectTrigger>
                <SelectContent>
                    {indicators.map((ind) => (
                        <SelectItem key={ind.value} value={ind.value}>
                            {ind.label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>

            <Select
                value={rule.operator}
                onValueChange={(v) =>
                    onChange({
                        ...rule,
                        operator: v,
                        value: v === 'between' ? [0, 1] : typeof rule.value === 'number' ? rule.value : 0,
                    })
                }
            >
                <SelectTrigger className="w-28">
                    <SelectValue placeholder="Op" />
                </SelectTrigger>
                <SelectContent>
                    {operators.map((op) => (
                        <SelectItem key={op.value} value={op.value}>
                            {op.label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>

            {isBetween ? (
                <div className="flex items-center gap-1">
                    <Input
                        type="number"
                        step="any"
                        className="w-20"
                        value={Array.isArray(rule.value) ? rule.value[0] : 0}
                        onChange={(e) =>
                            onChange({
                                ...rule,
                                value: [parseFloat(e.target.value) || 0, Array.isArray(rule.value) ? rule.value[1] : 1],
                            })
                        }
                    />
                    <span className="text-muted-foreground">—</span>
                    <Input
                        type="number"
                        step="any"
                        className="w-20"
                        value={Array.isArray(rule.value) ? rule.value[1] : 1}
                        onChange={(e) =>
                            onChange({
                                ...rule,
                                value: [Array.isArray(rule.value) ? rule.value[0] : 0, parseFloat(e.target.value) || 0],
                            })
                        }
                    />
                </div>
            ) : (
                <Input
                    type="number"
                    step="any"
                    className="w-24"
                    value={typeof rule.value === 'number' ? rule.value : 0}
                    onChange={(e) => onChange({ ...rule, value: parseFloat(e.target.value) || 0 })}
                />
            )}

            <Button variant="ghost" size="icon" onClick={onRemove}>
                <Trash2 className="size-4" />
            </Button>
        </div>
    );
}
```

**Step 4: Create ConditionGroup component**

Create `web/resources/js/components/strategy/condition-group.tsx`:

```tsx
import { Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { RuleRow } from './rule-row';
import type { ConditionGroup as ConditionGroupType, StrategyRule } from '@/types/models';

interface ConditionGroupProps {
    group: ConditionGroupType;
    index: number;
    onChange: (group: ConditionGroupType) => void;
    onRemove: () => void;
}

const emptyRule: StrategyRule = { indicator: 'abs_move_pct', operator: '>', value: 0 };

export function ConditionGroup({ group, index, onChange, onRemove }: ConditionGroupProps) {
    function updateRule(ruleIndex: number, rule: StrategyRule) {
        const rules = [...group.rules];
        rules[ruleIndex] = rule;
        onChange({ ...group, rules });
    }

    function removeRule(ruleIndex: number) {
        onChange({ ...group, rules: group.rules.filter((_, i) => i !== ruleIndex) });
    }

    function addRule() {
        onChange({ ...group, rules: [...group.rules, { ...emptyRule }] });
    }

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between pb-3">
                <CardTitle className="text-sm">Condition Group {index + 1}</CardTitle>
                <div className="flex items-center gap-2">
                    <Select
                        value={group.type}
                        onValueChange={(v) => onChange({ ...group, type: v as 'AND' | 'OR' })}
                    >
                        <SelectTrigger className="w-24">
                            <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="AND">AND</SelectItem>
                            <SelectItem value="OR">OR</SelectItem>
                        </SelectContent>
                    </Select>
                    <Button variant="ghost" size="sm" onClick={onRemove}>
                        Remove
                    </Button>
                </div>
            </CardHeader>
            <CardContent className="space-y-2">
                {group.rules.map((rule, i) => (
                    <RuleRow
                        key={i}
                        rule={rule}
                        onChange={(r) => updateRule(i, r)}
                        onRemove={() => removeRule(i)}
                    />
                ))}
                <Button variant="outline" size="sm" onClick={addRule}>
                    <Plus className="mr-1 size-3" /> Add Rule
                </Button>
            </CardContent>
        </Card>
    );
}
```

**Step 5: Create ActionConfig component**

Create `web/resources/js/components/strategy/action-config.tsx`:

```tsx
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { StrategyAction } from '@/types/models';

interface ActionConfigProps {
    action: StrategyAction;
    onChange: (action: StrategyAction) => void;
}

export function ActionConfig({ action, onChange }: ActionConfigProps) {
    return (
        <Card>
            <CardHeader className="pb-3">
                <CardTitle className="text-sm">Action (ALORS)</CardTitle>
            </CardHeader>
            <CardContent>
                <div className="grid gap-4 sm:grid-cols-2">
                    <div>
                        <Label>Signal</Label>
                        <Select value={action.signal} onValueChange={(v) => onChange({ ...action, signal: v as 'buy' | 'sell' })}>
                            <SelectTrigger><SelectValue /></SelectTrigger>
                            <SelectContent>
                                <SelectItem value="buy">Buy</SelectItem>
                                <SelectItem value="sell">Sell</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    <div>
                        <Label>Outcome</Label>
                        <Select value={action.outcome} onValueChange={(v) => onChange({ ...action, outcome: v as 'UP' | 'DOWN' })}>
                            <SelectTrigger><SelectValue /></SelectTrigger>
                            <SelectContent>
                                <SelectItem value="UP">UP</SelectItem>
                                <SelectItem value="DOWN">DOWN</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    <div>
                        <Label>Size (USDC)</Label>
                        <Input
                            type="number"
                            min={1}
                            value={action.size_usdc}
                            onChange={(e) => onChange({ ...action, size_usdc: parseFloat(e.target.value) || 0 })}
                        />
                    </div>
                    <div>
                        <Label>Order Type</Label>
                        <Select value={action.order_type} onValueChange={(v) => onChange({ ...action, order_type: v as 'market' | 'limit' })}>
                            <SelectTrigger><SelectValue /></SelectTrigger>
                            <SelectContent>
                                <SelectItem value="market">Market</SelectItem>
                                <SelectItem value="limit">Limit</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
```

**Step 6: Create RiskConfig component**

Create `web/resources/js/components/strategy/risk-config.tsx`:

```tsx
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { StrategyRisk } from '@/types/models';

interface RiskConfigProps {
    risk: StrategyRisk;
    onChange: (risk: StrategyRisk) => void;
}

export function RiskConfig({ risk, onChange }: RiskConfigProps) {
    return (
        <Card>
            <CardHeader className="pb-3">
                <CardTitle className="text-sm">Risk Management</CardTitle>
            </CardHeader>
            <CardContent>
                <div className="grid gap-4 sm:grid-cols-2">
                    <div>
                        <Label>Stop Loss (%)</Label>
                        <Input
                            type="number"
                            min={0}
                            max={100}
                            value={risk.stoploss_pct}
                            onChange={(e) => onChange({ ...risk, stoploss_pct: parseFloat(e.target.value) || 0 })}
                        />
                    </div>
                    <div>
                        <Label>Take Profit (%)</Label>
                        <Input
                            type="number"
                            min={0}
                            max={100}
                            value={risk.take_profit_pct}
                            onChange={(e) => onChange({ ...risk, take_profit_pct: parseFloat(e.target.value) || 0 })}
                        />
                    </div>
                    <div>
                        <Label>Max Position (USDC)</Label>
                        <Input
                            type="number"
                            min={1}
                            value={risk.max_position_usdc}
                            onChange={(e) => onChange({ ...risk, max_position_usdc: parseFloat(e.target.value) || 0 })}
                        />
                    </div>
                    <div>
                        <Label>Max Trades per Slot</Label>
                        <Input
                            type="number"
                            min={1}
                            max={100}
                            value={risk.max_trades_per_slot}
                            onChange={(e) => onChange({ ...risk, max_trades_per_slot: parseInt(e.target.value) || 1 })}
                        />
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
```

**Step 7: Create FormBuilder component (orchestrator)**

Create `web/resources/js/components/strategy/form-builder.tsx`:

```tsx
import { Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ConditionGroup } from './condition-group';
import { ActionConfig } from './action-config';
import { RiskConfig } from './risk-config';
import type { FormModeGraph, ConditionGroup as ConditionGroupType } from '@/types/models';

interface FormBuilderProps {
    graph: FormModeGraph;
    onChange: (graph: FormModeGraph) => void;
}

const emptyGroup: ConditionGroupType = {
    type: 'AND',
    rules: [{ indicator: 'abs_move_pct', operator: '>', value: 0 }],
};

export function FormBuilder({ graph, onChange }: FormBuilderProps) {
    function updateCondition(index: number, group: ConditionGroupType) {
        const conditions = [...graph.conditions];
        conditions[index] = group;
        onChange({ ...graph, conditions });
    }

    function removeCondition(index: number) {
        onChange({ ...graph, conditions: graph.conditions.filter((_, i) => i !== index) });
    }

    function addCondition() {
        onChange({ ...graph, conditions: [...graph.conditions, { ...emptyGroup, rules: [...emptyGroup.rules] }] });
    }

    return (
        <div className="space-y-4">
            <div>
                <h3 className="mb-2 text-sm font-medium">Conditions (SI)</h3>
                <div className="space-y-3">
                    {graph.conditions.map((group, i) => (
                        <ConditionGroup
                            key={i}
                            group={group}
                            index={i}
                            onChange={(g) => updateCondition(i, g)}
                            onRemove={() => removeCondition(i)}
                        />
                    ))}
                </div>
                <Button variant="outline" className="mt-3" onClick={addCondition}>
                    <Plus className="mr-1 size-4" /> Add Condition Group
                </Button>
            </div>

            <ActionConfig
                action={graph.action}
                onChange={(action) => onChange({ ...graph, action })}
            />

            <RiskConfig
                risk={graph.risk}
                onChange={(risk) => onChange({ ...graph, risk })}
            />
        </div>
    );
}
```

**Step 8: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds.

**Step 9: Commit**

```bash
git add web/resources/js/components/strategy/ web/resources/js/types/models.ts
git commit -m "feat(strategy): add form builder components (conditions, action, risk)"
```

---

## Task 5: Strategy Form Builder — Page Integration

**Files:**
- Modify: `web/resources/js/pages/strategies/create.tsx`
- Modify: `web/resources/js/pages/strategies/show.tsx`

**Step 1: Rewrite strategy create page with form builder**

Replace `web/resources/js/pages/strategies/create.tsx`:

```tsx
import { Head, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FormBuilder } from '@/components/strategy/form-builder';
import type { BreadcrumbItem } from '@/types';
import type { FormModeGraph } from '@/types/models';
import { index, create, store } from '@/actions/App/Http/Controllers/StrategyController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: index.url() },
    { title: 'Create', href: create.url() },
];

const defaultGraph: FormModeGraph = {
    mode: 'form',
    conditions: [
        {
            type: 'AND',
            rules: [{ indicator: 'abs_move_pct', operator: '>', value: 3.0 }],
        },
    ],
    action: {
        signal: 'buy',
        outcome: 'UP',
        size_mode: 'fixed',
        size_usdc: 50,
        order_type: 'market',
    },
    risk: {
        stoploss_pct: 30,
        take_profit_pct: 80,
        max_position_usdc: 200,
        max_trades_per_slot: 1,
    },
};

export default function StrategiesCreate() {
    const { data, setData, post, processing, errors } = useForm({
        name: '',
        description: '',
        mode: 'form' as 'form' | 'node',
        graph: defaultGraph as FormModeGraph,
    });

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        post(store.url());
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Create Strategy" />
            <div className="mx-auto max-w-3xl p-6">
                <h1 className="mb-6 text-2xl font-bold">Create Strategy</h1>
                <form onSubmit={handleSubmit} className="space-y-6">
                    <div className="grid gap-4 sm:grid-cols-2">
                        <div>
                            <Label htmlFor="name">Name</Label>
                            <Input
                                id="name"
                                value={data.name}
                                onChange={(e) => setData('name', e.target.value)}
                            />
                            {errors.name && <p className="mt-1 text-sm text-destructive">{errors.name}</p>}
                        </div>
                        <div>
                            <Label htmlFor="description">Description</Label>
                            <Textarea
                                id="description"
                                value={data.description}
                                onChange={(e) => setData('description', e.target.value)}
                                rows={1}
                            />
                        </div>
                    </div>

                    <Tabs
                        value={data.mode}
                        onValueChange={(v) => setData('mode', v as 'form' | 'node')}
                    >
                        <TabsList>
                            <TabsTrigger value="form">Form Builder</TabsTrigger>
                            <TabsTrigger value="node" disabled>
                                Node Editor (coming soon)
                            </TabsTrigger>
                        </TabsList>
                        <TabsContent value="form" className="mt-4">
                            <FormBuilder
                                graph={data.graph}
                                onChange={(graph) => setData('graph', graph)}
                            />
                        </TabsContent>
                    </Tabs>

                    <div className="flex justify-end">
                        <Button type="submit" disabled={processing}>
                            Create Strategy
                        </Button>
                    </div>
                </form>
            </div>
        </AppLayout>
    );
}
```

**Step 2: Add strategy graph display to show page**

Update `web/resources/js/pages/strategies/show.tsx` — add a "Strategy Rules" section in the Configuration card that displays the graph conditions, action, and risk in a readable format. Replace the Configuration `<dl>` section:

```tsx
// Inside the Configuration card, after the Mode/Status dl, add:
{strategy.graph?.mode === 'form' && strategy.graph?.conditions && (
    <div className="mt-4 space-y-2">
        <h3 className="text-sm font-medium">Conditions</h3>
        {(strategy.graph.conditions as Array<{ type: string; rules: Array<{ indicator: string; operator: string; value: number | number[] }> }>).map((group, i) => (
            <div key={i} className="rounded border p-2 text-xs">
                <span className="font-medium">{group.type}</span>
                {group.rules.map((rule, j) => (
                    <div key={j} className="ml-2 text-muted-foreground">
                        {rule.indicator} {rule.operator}{' '}
                        {Array.isArray(rule.value) ? rule.value.join(' — ') : rule.value}
                    </div>
                ))}
            </div>
        ))}
    </div>
)}
```

**Step 3: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds.

**Step 4: Run existing strategy tests**

Run: `cd web && php artisan test --compact --filter=StrategyControllerTest`
Expected: PASS.

**Step 5: Commit**

```bash
git add web/resources/js/pages/strategies/create.tsx web/resources/js/pages/strategies/show.tsx
git commit -m "feat(strategy): integrate form builder into create page with tabs"
```

---

## Task 6: Strategy Node Editor — React Flow

**Files:**
- Create: `web/resources/js/components/strategy/nodes/input-node.tsx`
- Create: `web/resources/js/components/strategy/nodes/comparator-node.tsx`
- Create: `web/resources/js/components/strategy/nodes/logic-node.tsx`
- Create: `web/resources/js/components/strategy/nodes/action-node.tsx`
- Create: `web/resources/js/components/strategy/nodes/indicator-node.tsx`
- Create: `web/resources/js/components/strategy/node-editor.tsx`
- Modify: `web/resources/js/pages/strategies/create.tsx`
- Modify: `web/resources/js/types/models.ts`

**Step 1: Add node mode types**

Add to `web/resources/js/types/models.ts`:

```typescript
// Strategy graph types for node mode
export interface GraphNode {
    id: string;
    type: 'input' | 'indicator' | 'comparator' | 'logic' | 'action';
    data: Record<string, unknown>;
    position?: { x: number; y: number };
}

export interface GraphEdge {
    source: string;
    target: string;
}

export interface NodeModeGraph {
    mode: 'node';
    nodes: GraphNode[];
    edges: GraphEdge[];
}
```

**Step 2: Create InputNode component**

Create `web/resources/js/components/strategy/nodes/input-node.tsx`:

```tsx
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { indicators } from '../indicator-options';

export function InputNode({ data, id }: NodeProps) {
    return (
        <div className="rounded-lg border bg-background p-3 shadow-sm">
            <div className="mb-1 text-xs font-medium text-muted-foreground">Input</div>
            <Select
                value={data.field as string}
                onValueChange={(v) => data.onUpdate?.(id, { field: v })}
            >
                <SelectTrigger className="h-7 w-36 text-xs">
                    <SelectValue placeholder="Field" />
                </SelectTrigger>
                <SelectContent>
                    {indicators.map((ind) => (
                        <SelectItem key={ind.value} value={ind.value}>
                            {ind.label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
            <Handle type="source" position={Position.Right} />
        </div>
    );
}
```

**Step 3: Create ComparatorNode component**

Create `web/resources/js/components/strategy/nodes/comparator-node.tsx`:

```tsx
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { operators } from '../indicator-options';

export function ComparatorNode({ data, id }: NodeProps) {
    return (
        <div className="rounded-lg border bg-background p-3 shadow-sm">
            <div className="mb-1 text-xs font-medium text-muted-foreground">Compare</div>
            <Handle type="target" position={Position.Left} />
            <div className="flex gap-1">
                <Select
                    value={data.operator as string}
                    onValueChange={(v) => data.onUpdate?.(id, { ...data, operator: v })}
                >
                    <SelectTrigger className="h-7 w-16 text-xs">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {operators.map((op) => (
                            <SelectItem key={op.value} value={op.value}>
                                {op.label}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <Input
                    type="number"
                    step="any"
                    className="h-7 w-16 text-xs"
                    value={data.value as number}
                    onChange={(e) => data.onUpdate?.(id, { ...data, value: parseFloat(e.target.value) || 0 })}
                />
            </div>
            <Handle type="source" position={Position.Right} />
        </div>
    );
}
```

**Step 4: Create LogicNode component**

Create `web/resources/js/components/strategy/nodes/logic-node.tsx`:

```tsx
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

export function LogicNode({ data, id }: NodeProps) {
    return (
        <div className="rounded-lg border border-blue-200 bg-blue-50 p-3 shadow-sm dark:border-blue-800 dark:bg-blue-950">
            <div className="mb-1 text-xs font-medium text-muted-foreground">Logic</div>
            <Handle type="target" position={Position.Left} />
            <Select
                value={data.operator as string}
                onValueChange={(v) => data.onUpdate?.(id, { operator: v })}
            >
                <SelectTrigger className="h-7 w-20 text-xs">
                    <SelectValue />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="AND">AND</SelectItem>
                    <SelectItem value="OR">OR</SelectItem>
                </SelectContent>
            </Select>
            <Handle type="source" position={Position.Right} />
        </div>
    );
}
```

**Step 5: Create IndicatorNode component**

Create `web/resources/js/components/strategy/nodes/indicator-node.tsx`:

```tsx
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { indicators } from '../indicator-options';

const fns = ['EMA', 'SMA', 'RSI'] as const;

export function IndicatorNode({ data, id }: NodeProps) {
    return (
        <div className="rounded-lg border border-purple-200 bg-purple-50 p-3 shadow-sm dark:border-purple-800 dark:bg-purple-950">
            <div className="mb-1 text-xs font-medium text-muted-foreground">Indicator</div>
            <div className="space-y-1">
                <Select
                    value={data.fn as string}
                    onValueChange={(v) => data.onUpdate?.(id, { ...data, fn: v })}
                >
                    <SelectTrigger className="h-7 w-24 text-xs">
                        <SelectValue placeholder="Fn" />
                    </SelectTrigger>
                    <SelectContent>
                        {fns.map((fn) => (
                            <SelectItem key={fn} value={fn}>{fn}</SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <div className="flex gap-1">
                    <Input
                        type="number"
                        min={1}
                        className="h-7 w-16 text-xs"
                        placeholder="Period"
                        value={data.period as number}
                        onChange={(e) => data.onUpdate?.(id, { ...data, period: parseInt(e.target.value) || 1 })}
                    />
                    <Select
                        value={data.field as string}
                        onValueChange={(v) => data.onUpdate?.(id, { ...data, field: v })}
                    >
                        <SelectTrigger className="h-7 w-24 text-xs">
                            <SelectValue placeholder="Field" />
                        </SelectTrigger>
                        <SelectContent>
                            {indicators.map((ind) => (
                                <SelectItem key={ind.value} value={ind.value}>{ind.label}</SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
            </div>
            <Handle type="source" position={Position.Right} />
        </div>
    );
}
```

**Step 6: Create ActionNode component**

Create `web/resources/js/components/strategy/nodes/action-node.tsx`:

```tsx
import { Handle, Position, type NodeProps } from '@xyflow/react';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

export function ActionNode({ data, id }: NodeProps) {
    return (
        <div className="rounded-lg border border-green-200 bg-green-50 p-3 shadow-sm dark:border-green-800 dark:bg-green-950">
            <div className="mb-1 text-xs font-medium text-muted-foreground">Action</div>
            <Handle type="target" position={Position.Left} />
            <div className="flex gap-1">
                <Select
                    value={data.signal as string}
                    onValueChange={(v) => data.onUpdate?.(id, { ...data, signal: v })}
                >
                    <SelectTrigger className="h-7 w-16 text-xs">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="buy">Buy</SelectItem>
                        <SelectItem value="sell">Sell</SelectItem>
                    </SelectContent>
                </Select>
                <Select
                    value={data.outcome as string}
                    onValueChange={(v) => data.onUpdate?.(id, { ...data, outcome: v })}
                >
                    <SelectTrigger className="h-7 w-16 text-xs">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="UP">UP</SelectItem>
                        <SelectItem value="DOWN">DOWN</SelectItem>
                    </SelectContent>
                </Select>
                <Input
                    type="number"
                    className="h-7 w-16 text-xs"
                    placeholder="USDC"
                    value={data.size_usdc as number}
                    onChange={(e) => data.onUpdate?.(id, { ...data, size_usdc: parseFloat(e.target.value) || 0 })}
                />
            </div>
        </div>
    );
}
```

**Step 7: Create NodeEditor orchestrator component**

Create `web/resources/js/components/strategy/node-editor.tsx`:

```tsx
import { useCallback, useMemo } from 'react';
import {
    ReactFlow,
    Background,
    Controls,
    addEdge,
    useNodesState,
    useEdgesState,
    type Connection,
    type Edge,
    type Node,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Button } from '@/components/ui/button';
import { InputNode } from './nodes/input-node';
import { ComparatorNode } from './nodes/comparator-node';
import { LogicNode } from './nodes/logic-node';
import { ActionNode } from './nodes/action-node';
import { IndicatorNode } from './nodes/indicator-node';
import type { NodeModeGraph } from '@/types/models';

interface NodeEditorProps {
    graph: NodeModeGraph;
    onChange: (graph: NodeModeGraph) => void;
}

let nodeId = 100;
function nextId() {
    return `n${++nodeId}`;
}

export function NodeEditor({ graph, onChange }: NodeEditorProps) {
    const initialNodes: Node[] = graph.nodes.map((n) => ({
        id: n.id,
        type: n.type,
        position: n.position ?? { x: 0, y: 0 },
        data: { ...n.data, onUpdate: handleNodeUpdate },
    }));

    const initialEdges: Edge[] = graph.edges.map((e, i) => ({
        id: `e${i}`,
        source: e.source,
        target: e.target,
    }));

    const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

    function handleNodeUpdate(id: string, data: Record<string, unknown>) {
        setNodes((nds) =>
            nds.map((n) => (n.id === id ? { ...n, data: { ...data, onUpdate: handleNodeUpdate } } : n)),
        );
    }

    const onConnect = useCallback(
        (connection: Connection) => setEdges((eds) => addEdge(connection, eds)),
        [setEdges],
    );

    // Sync back to parent on any change
    const syncGraph = useCallback(() => {
        onChange({
            mode: 'node',
            nodes: nodes.map((n) => ({
                id: n.id,
                type: n.type as NodeModeGraph['nodes'][0]['type'],
                data: Object.fromEntries(Object.entries(n.data).filter(([k]) => k !== 'onUpdate')),
                position: n.position,
            })),
            edges: edges.map((e) => ({ source: e.source, target: e.target })),
        });
    }, [nodes, edges, onChange]);

    const nodeTypes = useMemo(
        () => ({
            input: InputNode,
            comparator: ComparatorNode,
            logic: LogicNode,
            action: ActionNode,
            indicator: IndicatorNode,
        }),
        [],
    );

    function addNode(type: string, data: Record<string, unknown>) {
        const id = nextId();
        setNodes((nds) => [
            ...nds,
            {
                id,
                type,
                position: { x: Math.random() * 400, y: Math.random() * 300 },
                data: { ...data, onUpdate: handleNodeUpdate },
            },
        ]);
    }

    return (
        <div className="space-y-2">
            <div className="flex flex-wrap gap-2">
                <Button variant="outline" size="sm" onClick={() => addNode('input', { field: 'abs_move_pct' })}>
                    + Input
                </Button>
                <Button variant="outline" size="sm" onClick={() => addNode('indicator', { fn: 'EMA', period: 20, field: 'mid_up' })}>
                    + Indicator
                </Button>
                <Button variant="outline" size="sm" onClick={() => addNode('comparator', { operator: '>', value: 0 })}>
                    + Compare
                </Button>
                <Button variant="outline" size="sm" onClick={() => addNode('logic', { operator: 'AND' })}>
                    + Logic
                </Button>
                <Button variant="outline" size="sm" onClick={() => addNode('action', { signal: 'buy', outcome: 'UP', size_usdc: 50 })}>
                    + Action
                </Button>
                <Button size="sm" onClick={syncGraph}>
                    Save Graph
                </Button>
            </div>
            <div className="h-[500px] rounded-lg border">
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
```

**Step 8: Enable node editor tab in create page**

In `web/resources/js/pages/strategies/create.tsx`, remove `disabled` from the node editor TabsTrigger and add the TabsContent for node mode:

```tsx
// Import the NodeEditor
import { NodeEditor } from '@/components/strategy/node-editor';
import type { FormModeGraph, NodeModeGraph } from '@/types/models';

// Update form data type to support both modes
const { data, setData, post, processing, errors } = useForm({
    name: '',
    description: '',
    mode: 'form' as 'form' | 'node',
    graph: defaultGraph as FormModeGraph | NodeModeGraph,
});

// The default node graph
const defaultNodeGraph: NodeModeGraph = {
    mode: 'node',
    nodes: [
        { id: 'n1', type: 'input', data: { field: 'abs_move_pct' }, position: { x: 50, y: 100 } },
        { id: 'n2', type: 'comparator', data: { operator: '>', value: 3.0 }, position: { x: 300, y: 100 } },
        { id: 'n3', type: 'action', data: { signal: 'buy', outcome: 'UP', size_usdc: 50 }, position: { x: 550, y: 100 } },
    ],
    edges: [
        { source: 'n1', target: 'n2' },
        { source: 'n2', target: 'n3' },
    ],
};

// In the Tabs onValueChange, switch the graph structure:
onValueChange={(v) => {
    const mode = v as 'form' | 'node';
    setData({
        ...data,
        mode,
        graph: mode === 'form' ? defaultGraph : defaultNodeGraph,
    });
}}

// Add the TabsContent for node mode (remove disabled from TabsTrigger):
<TabsTrigger value="node">Node Editor</TabsTrigger>

<TabsContent value="node" className="mt-4">
    <NodeEditor
        graph={data.graph as NodeModeGraph}
        onChange={(graph) => setData('graph', graph)}
    />
</TabsContent>
```

**Step 9: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds.

**Step 10: Commit**

```bash
git add web/resources/js/components/strategy/nodes/ web/resources/js/components/strategy/node-editor.tsx web/resources/js/pages/strategies/create.tsx web/resources/js/types/models.ts
git commit -m "feat(strategy): add React Flow node editor with custom node types"
```

---

## Task 7: Backtest Page — Trigger Form + Charts

**Files:**
- Modify: `web/resources/js/pages/strategies/show.tsx`
- Modify: `web/resources/js/pages/backtests/show.tsx`
- Create: `web/resources/js/components/charts/pnl-chart.tsx`
- Modify: `web/resources/js/types/models.ts`

**Step 1: Add backtest trigger form to strategy show page**

In `web/resources/js/pages/strategies/show.tsx`, add a backtest trigger section after the "Recent Backtests" card. Use Inertia's `useForm` to POST to the backtest run endpoint:

```tsx
// Add imports
import { Head, router, useForm } from '@inertiajs/react';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { run as runBacktest } from '@/actions/App/Http/Controllers/BacktestController';

// Inside the component, before the return:
const backtestForm = useForm({
    date_from: '',
    date_to: '',
    market_filter: [] as string[],
});

function handleBacktest(e: React.FormEvent) {
    e.preventDefault();
    backtestForm.post(runBacktest.url(strategy.id));
}

// Add this section after the "Recent Backtests" card:
<div className="mt-6 rounded-lg border border-sidebar-border p-4">
    <h2 className="mb-3 font-semibold">Run Backtest</h2>
    <form onSubmit={handleBacktest} className="flex flex-wrap items-end gap-3">
        <div>
            <Label htmlFor="date_from">From</Label>
            <Input
                id="date_from"
                type="date"
                value={backtestForm.data.date_from}
                onChange={(e) => backtestForm.setData('date_from', e.target.value)}
            />
        </div>
        <div>
            <Label htmlFor="date_to">To</Label>
            <Input
                id="date_to"
                type="date"
                value={backtestForm.data.date_to}
                onChange={(e) => backtestForm.setData('date_to', e.target.value)}
            />
        </div>
        <Button type="submit" disabled={backtestForm.processing}>
            Run Backtest
        </Button>
    </form>
    {backtestForm.errors.date_from && (
        <p className="mt-1 text-sm text-destructive">{backtestForm.errors.date_from}</p>
    )}
    {backtestForm.errors.date_to && (
        <p className="mt-1 text-sm text-destructive">{backtestForm.errors.date_to}</p>
    )}
</div>
```

**Step 2: Update BacktestResult type for chart data**

Add to `web/resources/js/types/models.ts`:

```typescript
export interface BacktestTrade {
    tick_index: number;
    side: 'buy' | 'sell';
    outcome: 'UP' | 'DOWN';
    entry_price: number;
    exit_price: number | null;
    pnl: number;
    cumulative_pnl: number;
}
```

Update the `BacktestResult` interface to include optional `result_detail`:
```typescript
export interface BacktestResult {
    // ... existing fields ...
    result_detail?: {
        trades?: BacktestTrade[];
    } | null;
}
```

**Step 3: Create PnL chart component**

Create `web/resources/js/components/charts/pnl-chart.tsx`:

```tsx
import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import type { BacktestTrade } from '@/types/models';

interface PnlChartProps {
    trades: BacktestTrade[];
}

export function PnlChart({ trades }: PnlChartProps) {
    const data = trades.map((t, i) => ({
        trade: i + 1,
        pnl: t.cumulative_pnl,
    }));

    return (
        <ResponsiveContainer width="100%" height={300}>
            <AreaChart data={data}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis dataKey="trade" tick={{ fontSize: 12 }} />
                <YAxis tick={{ fontSize: 12 }} tickFormatter={(v) => `$${v}`} />
                <Tooltip
                    formatter={(value: number) => [`$${value.toFixed(2)}`, 'Cumulative PnL']}
                    contentStyle={{ background: 'hsl(var(--background))', border: '1px solid hsl(var(--border))' }}
                />
                <Area
                    type="monotone"
                    dataKey="pnl"
                    stroke="hsl(var(--chart-1))"
                    fill="hsl(var(--chart-1))"
                    fillOpacity={0.2}
                />
            </AreaChart>
        </ResponsiveContainer>
    );
}
```

**Step 4: Add chart to backtest show page**

Update `web/resources/js/pages/backtests/show.tsx` to include the PnL chart below the metric cards:

```tsx
// Add import
import { PnlChart } from '@/components/charts/pnl-chart';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

// After the metrics grid, add:
{result.result_detail?.trades && result.result_detail.trades.length > 0 && (
    <Card className="mt-6">
        <CardHeader>
            <CardTitle>Cumulative PnL</CardTitle>
        </CardHeader>
        <CardContent>
            <PnlChart trades={result.result_detail.trades} />
        </CardContent>
    </Card>
)}
```

**Step 5: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds.

**Step 6: Run tests**

Run: `cd web && php artisan test --compact --filter=BacktestControllerTest`
Expected: PASS.

**Step 7: Commit**

```bash
git add web/resources/js/pages/strategies/show.tsx web/resources/js/pages/backtests/show.tsx web/resources/js/components/charts/ web/resources/js/types/models.ts
git commit -m "feat(backtest): add trigger form and PnL chart to backtest pages"
```

---

## Task 8: Wallets — Strategy Assignment UI

**Files:**
- Modify: `web/resources/js/pages/wallets/index.tsx`
- Modify: `web/resources/js/types/models.ts`

**Step 1: Update WalletController to pass available strategies**

Write test first:

```php
// Add to WalletControllerTest
it('wallet index includes available strategies', function () {
    $user = User::factory()->create();
    Strategy::factory()->for($user)->create(['name' => 'Test Strategy']);

    $this->actingAs($user)
        ->get('/wallets')
        ->assertInertia(fn ($page) => $page
            ->component('wallets/index')
            ->has('strategies', 1)
        );
});
```

**Step 2: Run test to verify it fails**

Run: `cd web && php artisan test --compact --filter="wallet index includes available strategies"`
Expected: FAIL.

**Step 3: Update WalletController::index to include strategies**

In `web/app/Http/Controllers/WalletController.php`, update `index()`:

```php
public function index(): Response
{
    $user = auth()->user();

    return Inertia::render('wallets/index', [
        'wallets' => $user->wallets()
            ->withCount('strategies')
            ->latest('created_at')
            ->paginate(20)
            ->through(fn ($w) => $w->only('id', 'label', 'address', 'balance_usdc', 'is_active', 'strategies_count')),
        'strategies' => $user->strategies()->select('id', 'name')->get(),
    ]);
}
```

**Step 4: Run test to verify it passes**

Run: `cd web && php artisan test --compact --filter="wallet index includes available strategies"`
Expected: PASS.

**Step 5: Update Wallet type to include strategy assignments**

Update `web/resources/js/types/models.ts` — the Wallet interface already has `strategies_count`. Add a simple strategy type for the dropdown:

```typescript
// Already exists, but ensure Wallet has strategies_count
export interface Wallet {
    id: number;
    label: string | null;
    address: string;
    balance_usdc: string;
    is_active: boolean;
    strategies_count?: number;
}
```

**Step 6: Add strategy assignment dialog to wallets page**

Rewrite `web/resources/js/pages/wallets/index.tsx` to add an "Assign Strategy" button for each wallet with a dropdown:

```tsx
import { Head, router, useForm } from '@inertiajs/react';
import { useState } from 'react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import type { BreadcrumbItem } from '@/types';
import type { Wallet } from '@/types/models';
import { index, store, destroy, assignStrategy, removeStrategy } from '@/actions/App/Http/Controllers/WalletController';

const breadcrumbs: BreadcrumbItem[] = [{ title: 'Wallets', href: index.url() }];

interface Props {
    wallets: Wallet[];
    strategies: Array<{ id: number; name: string }>;
}

export default function WalletsIndex({ wallets, strategies }: Props) {
    const createForm = useForm({ label: '' });
    const assignForm = useForm({ strategy_id: '', max_position_usdc: '100' });
    const [assigningWallet, setAssigningWallet] = useState<number | null>(null);

    function handleCreate(e: React.FormEvent) {
        e.preventDefault();
        createForm.post(store.url(), { onSuccess: () => createForm.reset() });
    }

    function handleAssign(walletId: number) {
        assignForm.post(assignStrategy.url(walletId), {
            onSuccess: () => {
                assignForm.reset();
                setAssigningWallet(null);
            },
        });
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Wallets" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Wallets</h1>

                <form onSubmit={handleCreate} className="mb-6 flex items-end gap-3">
                    <div>
                        <Label htmlFor="label">Label (optional)</Label>
                        <Input
                            id="label"
                            value={createForm.data.label}
                            onChange={(e) => createForm.setData('label', e.target.value)}
                            placeholder="My trading wallet"
                        />
                    </div>
                    <Button type="submit" disabled={createForm.processing}>
                        Generate Wallet
                    </Button>
                </form>

                <div className="space-y-3">
                    {wallets.length === 0 && (
                        <p className="text-muted-foreground">No wallets yet. Generate your first one above.</p>
                    )}
                    {wallets.map((wallet) => (
                        <div
                            key={wallet.id}
                            className="rounded-xl border border-sidebar-border/70 p-4 dark:border-sidebar-border"
                        >
                            <div className="flex items-center justify-between">
                                <div>
                                    <h3 className="font-semibold">{wallet.label || 'Unnamed Wallet'}</h3>
                                    <p className="font-mono text-xs text-muted-foreground">{wallet.address}</p>
                                    <p className="mt-1 text-sm text-muted-foreground">
                                        ${parseFloat(wallet.balance_usdc).toFixed(2)} USDC · {wallet.strategies_count} strateg{wallet.strategies_count === 1 ? 'y' : 'ies'}
                                    </p>
                                </div>
                                <div className="flex gap-2">
                                    <Dialog open={assigningWallet === wallet.id} onOpenChange={(open) => setAssigningWallet(open ? wallet.id : null)}>
                                        <DialogTrigger asChild>
                                            <Button variant="outline" size="sm">Assign Strategy</Button>
                                        </DialogTrigger>
                                        <DialogContent>
                                            <DialogHeader>
                                                <DialogTitle>Assign Strategy</DialogTitle>
                                            </DialogHeader>
                                            <div className="space-y-4">
                                                <div>
                                                    <Label>Strategy</Label>
                                                    <Select
                                                        value={assignForm.data.strategy_id}
                                                        onValueChange={(v) => assignForm.setData('strategy_id', v)}
                                                    >
                                                        <SelectTrigger>
                                                            <SelectValue placeholder="Select a strategy" />
                                                        </SelectTrigger>
                                                        <SelectContent>
                                                            {strategies.map((s) => (
                                                                <SelectItem key={s.id} value={String(s.id)}>
                                                                    {s.name}
                                                                </SelectItem>
                                                            ))}
                                                        </SelectContent>
                                                    </Select>
                                                </div>
                                                <div>
                                                    <Label>Max Position (USDC)</Label>
                                                    <Input
                                                        type="number"
                                                        min={1}
                                                        value={assignForm.data.max_position_usdc}
                                                        onChange={(e) => assignForm.setData('max_position_usdc', e.target.value)}
                                                    />
                                                </div>
                                                <Button
                                                    onClick={() => handleAssign(wallet.id)}
                                                    disabled={assignForm.processing || !assignForm.data.strategy_id}
                                                >
                                                    Assign
                                                </Button>
                                            </div>
                                        </DialogContent>
                                    </Dialog>
                                    <Button
                                        variant="destructive"
                                        size="sm"
                                        onClick={() => {
                                            if (confirm('Are you sure you want to delete this wallet?')) {
                                                router.delete(destroy.url(wallet.id));
                                            }
                                        }}
                                    >
                                        Delete
                                    </Button>
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
```

**Step 7: Run Pint + verify build**

Run: `cd web && vendor/bin/pint --dirty --format agent && npm run build`
Expected: Both pass.

**Step 8: Run all wallet tests**

Run: `cd web && php artisan test --compact --filter=WalletControllerTest`
Expected: PASS.

**Step 9: Commit**

```bash
git add web/app/Http/Controllers/WalletController.php web/resources/js/pages/wallets/index.tsx web/tests/Feature/WalletControllerTest.php
git commit -m "feat(wallets): add strategy assignment dialog with available strategies"
```

---

## Task 9: Billing — Upgrade Flow

**Files:**
- Modify: `web/resources/js/pages/billing/index.tsx`

**Step 1: Add subscribe buttons to billing page**

Update `web/resources/js/pages/billing/index.tsx` to add upgrade buttons that POST to the subscribe endpoint:

```tsx
import { Head, router } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Check } from 'lucide-react';
import type { BreadcrumbItem } from '@/types';
import { index, subscribe, portal } from '@/actions/App/Http/Controllers/BillingController';

const breadcrumbs: BreadcrumbItem[] = [{ title: 'Billing', href: index.url() }];

const plans = [
    {
        key: 'free',
        name: 'Free',
        price: '$0',
        period: 'forever',
        features: ['1 wallet', '2 strategies', '30-day backtest', '1 copy leader'],
    },
    {
        key: 'starter',
        name: 'Starter',
        price: '$29',
        period: '/mo',
        priceId: 'price_starter',
        features: ['5 wallets', '10 strategies', 'Full history backtest', '5 copy leaders', 'Revenue sharing'],
    },
    {
        key: 'pro',
        name: 'Pro',
        price: '$79',
        period: '/mo',
        priceId: 'price_pro',
        popular: true,
        features: ['25 wallets', 'Unlimited strategies', 'Full history backtest', 'Unlimited copy + be leader', 'Revenue sharing'],
    },
    {
        key: 'enterprise',
        name: 'Enterprise',
        price: '$249',
        period: '/mo',
        priceId: 'price_enterprise',
        features: ['Unlimited wallets', 'Unlimited strategies', 'Full history + API', 'Custom leader fees', 'Revenue sharing'],
    },
];

interface Props {
    plan: string;
    subscribed: boolean;
}

export default function BillingIndex({ plan, subscribed }: Props) {
    function handleSubscribe(priceId: string) {
        router.post(subscribe.url(), { price_id: priceId });
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Billing" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Billing</h1>

                <div className="mb-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                    {plans.map((p) => (
                        <Card
                            key={p.key}
                            className={`relative ${plan === p.key ? 'ring-2 ring-primary' : ''} ${p.popular ? 'border-primary' : ''}`}
                        >
                            {p.popular && (
                                <Badge className="absolute -top-2.5 left-1/2 -translate-x-1/2">
                                    Popular
                                </Badge>
                            )}
                            <CardHeader>
                                <CardTitle>{p.name}</CardTitle>
                                <div className="flex items-baseline gap-1">
                                    <span className="text-3xl font-bold">{p.price}</span>
                                    <span className="text-sm text-muted-foreground">{p.period}</span>
                                </div>
                            </CardHeader>
                            <CardContent>
                                <ul className="mb-4 space-y-2 text-sm">
                                    {p.features.map((f) => (
                                        <li key={f} className="flex items-center gap-2">
                                            <Check className="size-4 text-green-500" />
                                            {f}
                                        </li>
                                    ))}
                                </ul>
                                {plan === p.key ? (
                                    <Button variant="outline" className="w-full" disabled>
                                        Current Plan
                                    </Button>
                                ) : p.priceId ? (
                                    <Button
                                        className="w-full"
                                        variant={p.popular ? 'default' : 'outline'}
                                        onClick={() => handleSubscribe(p.priceId!)}
                                    >
                                        {plan === 'free' ? 'Get Started' : 'Upgrade'}
                                    </Button>
                                ) : null}
                            </CardContent>
                        </Card>
                    ))}
                </div>

                {subscribed && (
                    <Button variant="outline" onClick={() => router.post(portal.url())}>
                        Manage Subscription
                    </Button>
                )}
            </div>
        </AppLayout>
    );
}
```

**Step 2: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds.

**Step 3: Run billing tests**

Run: `cd web && php artisan test --compact --filter=BillingControllerTest`
Expected: PASS.

**Step 4: Commit**

```bash
git add web/resources/js/pages/billing/index.tsx
git commit -m "feat(billing): add plan comparison cards with subscribe buttons"
```

---

## Task 10: Final Verification + Cleanup

**Step 1: Run Pint on all modified PHP files**

Run: `cd web && vendor/bin/pint --dirty --format agent`

**Step 2: Verify full build**

Run: `cd web && npm run build`
Expected: Build succeeds without errors or warnings.

**Step 3: Run full test suite**

Run: `cd web && php artisan test --compact`
Expected: All tests pass.

**Step 4: Final commit if anything remains**

```bash
git add -A
git commit -m "chore: phase 8 cleanup and lint fixes"
```
