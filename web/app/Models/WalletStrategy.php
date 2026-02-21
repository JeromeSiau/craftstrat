<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\Pivot;

class WalletStrategy extends Pivot
{
    protected $table = 'wallet_strategies';

    public $incrementing = true;

    public $timestamps = false;

    protected $fillable = [
        'wallet_id',
        'strategy_id',
        'markets',
        'max_position_usdc',
        'is_running',
        'started_at',
    ];

    protected function casts(): array
    {
        return [
            'markets' => 'array',
            'max_position_usdc' => 'decimal:6',
            'is_running' => 'boolean',
            'started_at' => 'datetime',
        ];
    }

    public function wallet(): BelongsTo
    {
        return $this->belongsTo(Wallet::class);
    }

    public function strategy(): BelongsTo
    {
        return $this->belongsTo(Strategy::class);
    }
}
