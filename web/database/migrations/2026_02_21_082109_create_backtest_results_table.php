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
        Schema::create('backtest_results', function (Blueprint $table) {
            $table->id();
            $table->foreignId('user_id')->constrained();
            $table->foreignId('strategy_id')->constrained();
            $table->jsonb('market_filter')->nullable();
            $table->timestamp('date_from')->nullable();
            $table->timestamp('date_to')->nullable();
            $table->integer('total_trades')->nullable();
            $table->decimal('win_rate', 5, 4)->nullable();
            $table->decimal('total_pnl_usdc', 18, 6)->nullable();
            $table->decimal('max_drawdown', 5, 4)->nullable();
            $table->decimal('sharpe_ratio', 8, 4)->nullable();
            $table->jsonb('result_detail')->nullable();
            $table->timestamp('created_at')->useCurrent();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('backtest_results');
    }
};
