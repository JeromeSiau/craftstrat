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
            $table->decimal('reference_price', 10, 6)->nullable();
            $table->decimal('resolved_price', 10, 6)->nullable();
            $table->decimal('fill_slippage_bps', 10, 2)->nullable();
            $table->decimal('fill_slippage_pct', 10, 6)->nullable();
            $table->decimal('markout_price_60s', 10, 6)->nullable();
            $table->timestamp('markout_at_60s')->nullable();
            $table->decimal('markout_bps_60s', 10, 2)->nullable();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('trades', function (Blueprint $table) {
            $table->dropColumn([
                'reference_price',
                'resolved_price',
                'fill_slippage_bps',
                'fill_slippage_pct',
                'markout_price_60s',
                'markout_at_60s',
                'markout_bps_60s',
            ]);
        });
    }
};
