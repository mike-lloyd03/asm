# ASM

Simple tool for getting secrets out of AWS Secrets Manager.

## Usage
You must have the AWS v2 cli installed on your machine and have it configured for this tool to work.

### Search secrets by name
Returns a list of secrets that match the search term.

```bash
$ asm search mySecret

╭──────────────┬─────────────────────╮
│Name          │ Description         │
├──────────────┼─────────────────────┤
│dev/mySecret  │ Development secret  │
│prod/mySecret │ Production secret   │
╰──────────────┴─────────────────────╯

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

╭────────────────┬─────────────────────╮
│Name            │ Description         │
├────────────────┼─────────────────────┤
│dev/mySecret    │ Development secret  │
│prod/mySecret   │ Production secret   │
│dev/extraSecret │ Extra secret item   │
│prod/secretItem │ Also secret item    │
╰────────────────┴─────────────────────╯
```

### Create a secret
Opens the user's preferred text editor and creates a new secret from the editor's contents
 
```bash
$ asm create --description "This is a new secret" dev/mySecret
```

### Edit a secret
Opens the user's preferred text editor to allow editing the new secret
 
```bash
$ asm edit dev/mySecret
```

### Delete a secret
Deletes the specified secret. Asks for confirmation
 
```bash
$ asm delete dev/mySecret
Are you sure you want to delete secret '{}' [y/N]? 
```

### Describe a secret
Describes the specified secret.
 
```bash
$ asm describe dev/mySecret
```

```json
{
  "ARN": "arn:aws:secretsmanager:us-west-1:123456:secret:dev/mySecret-abc123",
  "CreatedDate": "2021-12-21T13:31:05.321000-08:00",
  "LastAccessedDate": "2023-01-04T16:00:00-08:00",
  "LastChangedDate": "2023-01-05T15:34:45.873000-08:00",
  "Name": "dev/mySecret",
  "Tags": [],
  "VersionIdsToStages": {
    "0beef02a-67f4-4e60-bcd9-00b946ef9dbf": [
      "AWSCURRENT"
    ],
    "f921c1ea-6bd0-4e19-b35c-788f01b52d67": [
      "AWSPREVIOUS"
    ]
  }
}
```
