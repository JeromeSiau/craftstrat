<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

class CopyRelationship extends Model
{
    /** @use HasFactory<\Database\Factories\CopyRelationshipFactory> */
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'follower_wallet_id',
        'watched_wallet_id',
        'size_mode',
        'size_value',
        'max_position_usdc',
        'markets_filter',
        'is_active',
    ];

    protected function casts(): array
    {
        return [
            'size_value' => 'decimal:6',
            'max_position_usdc' => 'decimal:6',
            'markets_filter' => 'array',
            'is_active' => 'boolean',
        ];
    }

    public function followerWallet(): BelongsTo
    {
        return $this->belongsTo(Wallet::class, 'follower_wallet_id');
    }

    public function watchedWallet(): BelongsTo
    {
        return $this->belongsTo(WatchedWallet::class);
    }

    public function copyTrades(): HasMany
    {
        return $this->hasMany(CopyTrade::class);
    }
}
