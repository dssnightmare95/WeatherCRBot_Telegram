# Etapa 1: Build de Rust
FROM rust:1.74 as builder

WORKDIR /app

# Copia solo los archivos necesarios para resolver dependencias
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() { println!("Dummy"); }' > src/main.rs
RUN cargo build --release
RUN rm -r src

# Ahora copiamos el código real y recompilamos
COPY src ./src
RUN cargo build --release

# Etapa 2: Imagen final mínima para producción
FROM debian:buster-slim

# Crear usuario no root
RUN useradd -m botuser

# Instalar dependencias mínimas si tu bot necesita (por ejemplo, para HTTPS con TLS)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copiar binario desde etapa builder
COPY --from=builder /app/target/release/WeatherCR /usr/local/bin/WeatherCR

# Establecer usuario
USER botuser

# Establecer directorio de trabajo
WORKDIR /app

# Copiar archivo .env si lo deseas para pruebas locales (no recomendado en prod)
# COPY .env .env

# Exponer el puerto si usas webhooks (ajústalo si es necesario)
EXPOSE 8080

# Ejecutar el bot
CMD ["WeatherCR"]
