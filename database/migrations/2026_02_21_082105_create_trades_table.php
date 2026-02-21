<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('trades', function (Blueprint $table) {
            $table->id();
            $table->foreignId('wallet_id')->constrained();
            $table->foreignId('strategy_id')->nullable()->constrained();
            $table->unsignedBigInteger('copy_relationship_id')->nullable();
            $table->string('market_id', 100)->nullable();
            $table->string('side', 10)->nullable();
            $table->string('outcome', 10)->nullable();
            $table->decimal('price', 10, 6)->nullable();
            $table->decimal('size_usdc', 18, 6)->nullable();
            $table->string('order_type', 20)->nullable();
            $table->string('status', 20)->default('pending');
            $table->string('polymarket_order_id')->nullable();
            $table->smallInteger('fee_bps')->nullable();
            $table->timestamp('executed_at')->nullable();
            $table->timestamp('created_at')->useCurrent();
            $table->index(['wallet_id', 'created_at'], 'idx_trades_wallet');
            $table->index('market_id', 'idx_trades_market');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('trades');
    }
};
