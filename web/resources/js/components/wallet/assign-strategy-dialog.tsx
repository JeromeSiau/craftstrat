import { useForm } from '@inertiajs/react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import InputError from '@/components/input-error';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { assignStrategy } from '@/actions/App/Http/Controllers/WalletController';

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
        max_position_usdc: '100',
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
