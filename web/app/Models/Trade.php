<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class Trade extends Model
{
    /** @use HasFactory<\Database\Factories\TradeFactory> */
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'wallet_id',
        'strategy_id',
        'copy_relationship_id',
        'market_id',
        'side',
        'outcome',
        'price',
        'size_usdc',
        'order_type',
        'status',
        'is_paper',
        'polymarket_order_id',
        'fee_bps',
        'executed_at',
    ];

    protected function casts(): array
    {
        return [
            'price' => 'decimal:6',
            'size_usdc' => 'decimal:6',
            'is_paper' => 'boolean',
            'fee_bps' => 'integer',
            'executed_at' => 'datetime',
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

    public function copyRelationship(): BelongsTo
    {
        return $this->belongsTo(CopyRelationship::class);
    }

    public function scopePaper(Builder $query): Builder
    {
        return $query->where('is_paper', true);
    }

    public function scopeLive(Builder $query): Builder
    {
        return $query->where('is_paper', false);
    }
}
