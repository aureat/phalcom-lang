acceptClosedHeaders( context: { headers: Headers } ) -> {}

brokenClosedRowWidth() {
    acceptClosedHeaders({
        headers: Headers {
            authorization: None,
        },
        path: "/extra",
    })
}
