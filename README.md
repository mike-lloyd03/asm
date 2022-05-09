# ASM

Simple tool for getting secrets out of AWS Secrets Manager.

## Usage
You must have the AWS v2 cli installed on your machine and have it configured for this tool to work.

### Search secrets by name
Returns a list of secrets that match the search term.

```bash
$ asm search mySecret

 Name          | Description
---------------+---------------------
 dev/mySecret  | Development secret
 prod/mySecret | Production secret

```

### Get secret ARN
Returns the ARN of a secret. If more than one secret matches the search term, the user will be prompted to select one.
```
$ asm get-arn dev/mySecret
arn:aws:secretsmanager:us-west-1:123456789101:secret:dev/mySecret-12345
```

### Get secret value
Returns the secret value of a secret. If more than one secret matches the search term, the user will be prompted to select one.
 
```bash
$ asm get-value dev/mySecret
{
  "value1": "json formatted secret",
  "value2": "in color too!",
}
```

### List all secrets
List all secrets along with their description
 
```bash
$ asm list

 Name            | Description
-----------------+--------------------
 dev/mySecret    | Development secret
 prod/mySecret   | Production secret
 dev/extraSecret | Extra secret item
 prod/secretItem | Also secret item

```

## Todo
- Better error handling
