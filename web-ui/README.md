# SuperRelay Web UI

Independent web UI components for SuperRelay API Gateway.

## Architecture

This directory contains web-based UI components that are deployed separately from the main Rust gateway binary, allowing for:

- Independent technology stack (Node.js/React/Vue vs Rust)
- Separate deployment and scaling
- UI team autonomy for updates and maintenance

## Components

### Swagger UI (`swagger-ui/`)

Interactive API documentation for SuperRelay JSON-RPC endpoints.

**Features:**
- Real-time API testing
- Complete ERC-4337 method documentation
- Request/response examples
- Authentication testing

**Usage:**
```bash
cd web-ui
npm install
npm run serve
```

Access at: http://localhost:9000/swagger-ui/

### Future Components

- Admin Dashboard
- Monitoring UI
- Policy Configuration UI
- Analytics Dashboard

## Development

### Prerequisites

- Node.js 16+
- npm or yarn

### Local Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

### Deployment

The web UI can be deployed independently using any static hosting solution:

- Nginx/Apache static hosting
- CDN deployment (CloudFront, CloudFlare)
- Docker container with nginx
- Kubernetes static file serving

### Configuration

Update `swagger-ui/openapi.json` to match your SuperRelay Gateway configuration:

```json
{
  "servers": [
    {
      "url": "https://your-gateway.example.com",
      "description": "Production SuperRelay Gateway"
    }
  ]
}
```

## API Integration

The web UI communicates with the SuperRelay Gateway via:

- JSON-RPC HTTP requests to the gateway endpoint
- Health check polling for status monitoring
- WebSocket connections for real-time updates (future)

## Security

When deploying in production:

1. Configure CORS properly in the gateway
2. Use HTTPS for all communications
3. Implement proper authentication headers
4. Rate limit UI requests appropriately

## Contributing

1. Follow the existing code style
2. Update OpenAPI specifications when adding new endpoints
3. Test UI changes across different browsers
4. Ensure mobile responsiveness