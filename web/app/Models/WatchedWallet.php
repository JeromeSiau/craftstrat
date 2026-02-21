<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

class WatchedWallet extends Model
{
    /** @use HasFactory<\Database\Factories\WatchedWalletFactory> */
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'address',
        'label',
        'follower_count',
        'win_rate',
        'total_pnl_usdc',
        'avg_slippage',
        'last_seen_at',
    ];

    protected function casts(): array
    {
        return [
            'follower_count' => 'integer',
            'win_rate' => 'decimal:4',
            'total_pnl_usdc' => 'decimal:6',
            'avg_slippage' => 'decimal:6',
            'last_seen_at' => 'datetime',
        ];
    }

    public function copyRelationships(): HasMany
    {
        return $this->hasMany(CopyRelationship::class);
    }
}
