type data Method {
    Get
    Post
    Put
    Patch
    Delete
}

type data Status {
    Ok
    Created
    NoContent
    BadRequest
    Unauthorized
    Forbidden
    NotFound
    Conflict
}

data Headers {
    authorization: Option<String>
}

data RawRequest {
    method: Method
    path: String
    headers: Headers
    rawBody: Bytes
}

data Response<S: Status, Body> {
    body: Body
    headers: Headers
}
