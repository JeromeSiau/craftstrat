export const MARKET_OPTIONS = [
    { label: 'BTC 15m', value: 'btc-updown-15m' },
    { label: 'BTC 5m', value: 'btc-updown-5m' },
    { label: 'ETH 15m', value: 'eth-updown-15m' },
    { label: 'SOL 15m', value: 'sol-updown-15m' },
    { label: 'XRP 15m', value: 'xrp-updown-15m' },
];

export const MARKET_LABEL_MAP: Record<string, string> = Object.fromEntries(
    MARKET_OPTIONS.map((m) => [m.value, m.label]),
);
