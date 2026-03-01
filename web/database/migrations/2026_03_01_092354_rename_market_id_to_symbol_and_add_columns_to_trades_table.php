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
        Schema::table('trades', function (Blueprint $table) {
            $table->renameColumn('market_id', 'symbol');
            $table->string('token_id', 100)->nullable()->after('symbol');
            $table->decimal('filled_price', 10, 6)->nullable()->after('fee_bps');
        });

        Schema::table('trades', function (Blueprint $table) {
            $table->renameIndex('idx_trades_market', 'idx_trades_symbol');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('trades', function (Blueprint $table) {
            $table->renameIndex('idx_trades_symbol', 'idx_trades_market');
        });

        Schema::table('trades', function (Blueprint $table) {
            $table->dropColumn(['token_id', 'filled_price']);
            $table->renameColumn('symbol', 'market_id');
        });
    }
};
