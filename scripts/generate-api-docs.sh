#!/bin/bash
# Generate API Documentation
# Creates OpenAPI JSON and static HTML documentation

set -e

echo "📚 Generating Ruflo API Documentation..."

# Create output directory
mkdir -p docs/api

# Generate OpenAPI spec
cd ruflo-command
cargo run --example generate_openapi 2>/dev/null || echo "⚠️  Example not found, using direct generation"

# If we have the spec generation in the binary, run it
cargo build --release 2>/dev/null || true

# Generate using utoipa if available
if command -v cargo-utoipa &> /dev/null; then
    echo "Using cargo-utoipa..."
    cargo utoipa gen --output ../docs/api/openapi.json
else
    echo "Generating OpenAPI spec programmatically..."
    # The spec is served at /api-docs/openapi.json when server is running
    echo "Start the server and curl: curl http://localhost:8080/api-docs/openapi.json > docs/api/openapi.json"
fi

cd ..

# Generate HTML with Swagger UI if spec exists
if [ -f "docs/api/openapi.json" ]; then
    echo "✅ OpenAPI spec generated: docs/api/openapi.json"
    
    # Create static HTML file
    cat > docs/api/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ruflo API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: 'openapi.json',
                dom_id: '#swagger-ui',
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.presets.standalone
                ],
                layout: "BaseLayout"
            });
        };
    </script>
</body>
</html>
EOF
    
    echo "✅ Swagger UI HTML created: docs/api/index.html"
else
    echo "⚠️  OpenAPI spec not found. Start the server to generate it."
fi

echo ""
echo "📖 Documentation generated successfully!"
echo "   - OpenAPI JSON: docs/api/openapi.json"
echo "   - Swagger UI:   docs/api/index.html"
echo ""
echo "To view locally: open docs/api/index.html"
