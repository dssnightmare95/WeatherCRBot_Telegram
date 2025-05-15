# Etapa 1: Build
FROM rust:latest as builder

# Crea directorio de trabajo
WORKDIR /app

# Copiar Cargo.toml y Cargo.lock
COPY WeatherCR/Cargo.toml WeatherCR/Cargo.lock ./

# Crear carpeta dummy para resolver dependencias
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -r src

# Copiar el resto del código fuente
COPY WeatherCR/src ./src

# Compilar en modo release
RUN cargo build --release

# Etapa 2: Imagen final
FROM debian:buster-slim

# Crear usuario no root
RUN useradd -m botuser

# Instalar certificados SSL si es necesario
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copiar el binario compilado desde el builder
COPY --from=builder /app/target/release/WeatherCR /usr/local/bin/WeatherCR

# Cambiar usuario
USER botuser

# Establecer directorio de trabajo
WORKDIR /app

# Exponer el puerto (si aplica)
EXPOSE 8080

# Comando por defecto
CMD ["WeatherCR"]
