FROM php:8.4-fpm AS base

# System deps
RUN apt-get update && apt-get install -y \
    git curl zip unzip libpq-dev libzip-dev libicu-dev \
    nginx supervisor \
    && docker-php-ext-install pdo_pgsql pgsql zip intl pcntl bcmath \
    && pecl install redis && docker-php-ext-enable redis \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Node.js 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Laravel installer (for scaffolding)
RUN composer global require laravel/installer

ENV PATH="/root/.composer/vendor/bin:${PATH}"

# Nginx config
COPY infra/nginx/default.conf /etc/nginx/sites-available/default

# Supervisor config (PHP-FPM + Nginx)
RUN printf '[supervisord]\nnodaemon=true\n\n[program:php-fpm]\ncommand=php-fpm\nautostart=true\nautorestart=true\n\n[program:nginx]\ncommand=nginx -g "daemon off;"\nautostart=true\nautorestart=true\n' \
    > /etc/supervisor/conf.d/app.conf

WORKDIR /var/www/html

EXPOSE 80

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/app.conf"]
