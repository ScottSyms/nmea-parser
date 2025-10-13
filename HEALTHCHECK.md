# Docker Health Check

The NMEA Parser CLI includes a built-in health check functionality that can be used for Docker container monitoring, Kubernetes liveness/readiness probes, and general service health validation.

## Health Check Features

The health check performs the following validations:

1. **NMEA Parsing Test** - Verifies the core NMEA parsing functionality
2. **S3 Connectivity** - Tests S3/MinIO connectivity if S3 is configured
3. **Memory Allocation** - Validates memory allocation capabilities
4. **JSON Serialization** - Tests JSON output functionality
5. **UUID v7 Generation** - Verifies UUID generation for record IDs

## Usage

### Command Line
```bash
# Basic health check
nmea-cli --health-check

# Health check with S3 validation
nmea-cli --health-check --s3-bucket my-bucket --s3-endpoint http://minio:9000
```

### Docker Health Check
```dockerfile
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD nmea-cli --health-check || exit 1
```

### Docker Compose
```yaml
services:
  nmea-parser:
    build: .
    healthcheck:
      test: ["CMD", "nmea-cli", "--health-check"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
```

### Kubernetes Probes
```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: nmea-parser
    image: nmea-parser:latest
    livenessProbe:
      exec:
        command:
        - nmea-cli
        - --health-check
      initialDelaySeconds: 30
      periodSeconds: 60
      timeoutSeconds: 10
      failureThreshold: 3
    readinessProbe:
      exec:
        command:
        - nmea-cli
        - --health-check
      initialDelaySeconds: 5
      periodSeconds: 30
      timeoutSeconds: 5
      failureThreshold: 2
```

## Exit Codes

- **0**: All health checks passed
- **1**: One or more health checks failed

## Health Check Output

```
🏥 NMEA Parser Health Check
==========================
✓ Testing NMEA parsing... PASSED
✓ Testing S3 connectivity... PASSED
✓ Testing memory allocation... PASSED
✓ Testing JSON serialization... PASSED
✓ Testing UUID v7 generation... PASSED
==========================
🎉 All health checks PASSED
```

## Environment Variables

The health check respects the same environment variables as the main application:

- `AWS_ACCESS_KEY_ID` - S3 access key
- `AWS_SECRET_ACCESS_KEY` - S3 secret key
- `S3_REGION` - S3 region (via --s3-region flag)
- `S3_ENDPOINT` - Custom S3 endpoint (via --s3-endpoint flag)

## Monitoring Integration

### Docker Swarm
```yaml
version: '3.8'
services:
  nmea-parser:
    image: nmea-parser:latest
    healthcheck:
      test: nmea-cli --health-check
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: any
```

### Prometheus Monitoring
You can expose health check metrics by parsing the exit code:

```bash
#!/bin/bash
# health-check-metrics.sh
if nmea-cli --health-check > /dev/null 2>&1; then
    echo "nmea_parser_health_check 1"
else
    echo "nmea_parser_health_check 0"
fi
```

## Troubleshooting

### Common Issues

1. **S3 Connection Failures**
   - Verify AWS credentials
   - Check network connectivity to S3 endpoint
   - Validate S3 bucket permissions

2. **Memory Allocation Failures**
   - Check container memory limits
   - Verify available system memory

3. **JSON Serialization Failures**
   - Usually indicates system instability
   - Check container resources

### Debug Mode
Add `--verbose` flag to get detailed error information:

```bash
nmea-cli --health-check --verbose
```

## CI/CD Integration

### GitHub Actions
```yaml
- name: Health Check
  run: |
    docker run --rm nmea-parser:latest nmea-cli --health-check
```

### GitLab CI
```yaml
health_check:
  script:
    - docker run --rm $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA nmea-cli --health-check
```