# =============================================================================
# Stage 1: Composer dependencies
# =============================================================================
FROM composer:latest AS composer-deps

RUN apk add --no-cache gmp-dev \
    && docker-php-ext-install bcmath gmp

WORKDIR /app
COPY web/composer.json web/composer.lock ./
RUN composer install --no-dev --no-scripts --no-autoloader --prefer-dist

COPY web/ .
RUN composer dump-autoload --optimize

# =============================================================================
# Stage 2: Frontend build
# =============================================================================
FROM node:22-alpine AS frontend

WORKDIR /app
COPY web/package.json web/package-lock.json ./
RUN npm ci

COPY web/ .
COPY --from=composer-deps /app/vendor ./vendor
RUN npm run build

# =============================================================================
# Stage 3: Production image
# =============================================================================
FROM php:8.4-fpm AS production

# System deps
RUN apt-get update && apt-get install -y \
    curl libpq-dev libzip-dev libicu-dev libgmp-dev \
    nginx supervisor cron \
    && docker-php-ext-install pdo_pgsql pgsql zip intl pcntl bcmath gmp \
    && pecl install redis && docker-php-ext-enable redis \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# PHP production config
RUN mv "$PHP_INI_DIR/php.ini-production" "$PHP_INI_DIR/php.ini"

# Nginx config
COPY infra/nginx/default.conf /etc/nginx/sites-available/default

# Supervisor config: PHP-FPM + Nginx + Queue Worker + Scheduler
COPY infra/docker/supervisord.conf /etc/supervisor/conf.d/app.conf

# Laravel scheduler cron
RUN echo "* * * * * cd /var/www/html && php artisan schedule:run >> /dev/null 2>&1" \
    | crontab -

WORKDIR /var/www/html

# Copy application code + built assets
COPY --from=composer-deps /app ./
COPY --from=frontend /app/public/build ./public/build

# Storage & cache directories with correct permissions
RUN chown -R www-data:www-data storage bootstrap/cache \
    && chmod -R 775 storage bootstrap/cache

EXPOSE 80

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/app.conf"]
