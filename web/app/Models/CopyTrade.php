<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class CopyTrade extends Model
{
    /** @use HasFactory<\Database\Factories\CopyTradeFactory> */
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'copy_relationship_id',
        'follower_trade_id',
        'leader_address',
        'leader_market_id',
        'leader_outcome',
        'leader_price',
        'leader_size_usdc',
        'leader_tx_hash',
        'follower_price',
        'slippage_pct',
        'status',
        'skip_reason',
        'detected_at',
        'executed_at',
    ];

    protected function casts(): array
    {
        return [
            'leader_price' => 'decimal:6',
            'leader_size_usdc' => 'decimal:6',
            'follower_price' => 'decimal:6',
            'slippage_pct' => 'decimal:6',
            'detected_at' => 'datetime',
            'executed_at' => 'datetime',
        ];
    }

    public function copyRelationship(): BelongsTo
    {
        return $this->belongsTo(CopyRelationship::class);
    }

    public function followerTrade(): BelongsTo
    {
        return $this->belongsTo(Trade::class, 'follower_trade_id');
    }
}
