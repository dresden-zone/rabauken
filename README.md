# Dresden Zone DNS Service


## API Schema


### Auth

- **/v1/auth/login** -> keycloak
- **/v1/auth/register** ->  keycloak
- **/v1/auth/logout** -> keycloak

### Zone

- **POST /v1/zone**
**Request**
```json
{
  "admin": "admin@dresden.zone",
  "name": "dresden.zone",
  "refresh": 3000,
  "retry": 3000,
  "expire": 3000,
  "minimum": 3000,
}
```

**Response**
```json
{
  "id": "zone-uuid"
}
```

- **GET /v1/zone-uuid/{uuid}**
**Response**
{
  "admin": "admin@dresden.zone",
  "name": "dresden.zone",
  "refresh": 3000,
  "retry": 3000,
  "expire": 3000,
  "minimum": 3000,
  "verified": true
}

- **DELETE /v1/zone-uuid/{uuid}**
- **PUT /v1/zone-uuid/{uuid}**

### Records

- **POST /v1/zone/{uuid-uuid}/record** 
**Request**
```json
{
  "type": "A",
  "name": "dns",  
  "address": "172.0.0.1"
  "ttl": 300
}
```

**Response**
```json
{
  "id": "record-uuid"
}
```

- **GET /v1/zone/{zone-uuid}/record** 
**Response**
```json
[
  {
    "id": "record-uuid"
    "type": "A",
    "name": "dns",  
    "address": "172.0.0.1"
    "ttl": 300
  },
  {
    "id": "record-uuid"
    "type": "A",
    "name": "api",  
    "address": "172.0.0.2"
    "ttl": 600
  }
]
```

- **DELETE /v1/zone/{zone-uuid}/record/{record-uuid}**
- **PUT /v1/zone/{zone-uuid}/record/{record-uuid}**
- **GET /v1/zone/{zone-uuid}/record/{record-uuid}**

