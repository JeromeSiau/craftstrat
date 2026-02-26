import { useForm } from '@inertiajs/react';
import { assignStrategy } from '@/actions/App/Http/Controllers/WalletController';
import InputError from '@/components/input-error';
import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { MARKET_OPTIONS } from '@/lib/constants';

interface AssignStrategyDialogProps {
    walletId: number;
    walletLabel: string | null;
    strategies: Array<{ id: number; name: string }>;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export default function AssignStrategyDialog({
    walletId,
    walletLabel,
    strategies,
    open,
    onOpenChange,
}: AssignStrategyDialogProps) {
    const form = useForm({
        strategy_id: '',
        markets: [] as string[],
        max_position_usdc: '100',
        is_paper: false,
    });

    function handleAssign(): void {
        form.post(assignStrategy.url(walletId), {
            onSuccess: () => {
                onOpenChange(false);
                form.reset();
            },
        });
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogTrigger asChild>
                <Button variant="outline" size="sm">
                    Assign Strategy
                </Button>
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>
                        Assign Strategy to {walletLabel || 'Unnamed Wallet'}
                    </DialogTitle>
                </DialogHeader>
                <div className="space-y-4 pt-2">
                    <div>
                        <Label htmlFor="strategy_id">Strategy</Label>
                        <Select
                            value={form.data.strategy_id}
                            onValueChange={(value) => form.setData('strategy_id', value)}
                        >
                            <SelectTrigger id="strategy_id" className="mt-1">
                                <SelectValue placeholder="Select a strategy" />
                            </SelectTrigger>
                            <SelectContent>
                                {strategies.map((strategy) => (
                                    <SelectItem key={strategy.id} value={String(strategy.id)}>
                                        {strategy.name}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <InputError message={form.errors.strategy_id} className="mt-1" />
                    </div>
                    <div>
                        <Label>Markets</Label>
                        <div className="mt-1.5 flex flex-wrap gap-1.5">
                            {MARKET_OPTIONS.map((m) => {
                                const isActive =
                                    form.data.markets.length === 0 ||
                                    form.data.markets.includes(m.value);
                                return (
                                    <button
                                        key={m.value}
                                        type="button"
                                        onClick={() => {
                                            const current = form.data.markets;
                                            let next: string[];
                                            if (current.length === 0) {
                                                next = [m.value];
                                            } else if (current.includes(m.value)) {
                                                next = current.filter((v) => v !== m.value);
                                            } else {
                                                next = [...current, m.value];
                                            }
                                            form.setData('markets', next);
                                        }}
                                        className={`rounded-md border px-2.5 py-1 text-xs font-medium transition-colors ${
                                            isActive
                                                ? 'border-primary bg-primary text-primary-foreground'
                                                : 'border-border bg-background text-muted-foreground hover:bg-accent'
                                        }`}
                                    >
                                        {m.label}
                                    </button>
                                );
                            })}
                        </div>
                        <p className="mt-1 text-xs text-muted-foreground">
                            {form.data.markets.length === 0
                                ? 'All markets'
                                : `${form.data.markets.length} selected`}
                        </p>
                    </div>
                    <div>
                        <Label htmlFor="max_position_usdc">Max Position (USDC)</Label>
                        <Input
                            id="max_position_usdc"
                            type="number"
                            min="1"
                            className="mt-1"
                            value={form.data.max_position_usdc}
                            onChange={(e) => form.setData('max_position_usdc', e.target.value)}
                        />
                        <InputError message={form.errors.max_position_usdc} className="mt-1" />
                    </div>
                    <div className="flex items-center justify-between">
                        <Label htmlFor="is_paper">Paper Trading</Label>
                        <Switch
                            id="is_paper"
                            checked={form.data.is_paper}
                            onCheckedChange={(checked) => form.setData('is_paper', checked)}
                        />
                    </div>
                    <Button
                        onClick={handleAssign}
                        disabled={form.processing || !form.data.strategy_id}
                        className="w-full"
                    >
                        Assign
                    </Button>
                </div>
            </DialogContent>
        </Dialog>
    );
}
