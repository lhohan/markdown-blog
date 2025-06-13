FROM ubuntu:latest

# Install necessary tools
RUN apt-get update && apt-get install -y ca-certificates

# Copy the executable
COPY target/release/blog-engine-shuttle /app/blog-engine-shuttle

# Set the working directory
WORKDIR /app

# Make the binary executable
RUN chmod +x blog-engine-shuttle

# Expose the port (if needed - adjust if your app uses a different port)
EXPOSE 8000

# Run the application
CMD ["./blog-engine-shuttle"]
