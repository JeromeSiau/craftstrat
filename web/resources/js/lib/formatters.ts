let uidCounter = 0;

export function uid(): string {
    uidCounter += 1;
    return `_${Date.now()}_${uidCounter}`;
}

export function safeParseFloat(value: string, fallback = 0): number {
    const num = value === '' ? fallback : parseFloat(value);
    return isNaN(num) ? fallback : num;
}

export function formatWinRate(winRate: string | null): string {
    if (!winRate) return '-';
    return `${(parseFloat(winRate) * 100).toFixed(1)}%`;
}

export function formatPnl(pnlUsdc: string | null): string {
    if (!pnlUsdc) return '-';
    return `$${parseFloat(pnlUsdc).toFixed(2)}`;
}

export function formatPercentage(value: string | null): string {
    if (!value) return '-';
    return `${(parseFloat(value) * 100).toFixed(1)}%`;
}

export function pnlColorClass(pnlUsdc: string | null): string {
    if (!pnlUsdc) return '';
    return parseFloat(pnlUsdc) >= 0 ? 'text-green-600' : 'text-red-600';
}
