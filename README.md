# axum_either

Accept and respond with one of multiple types in axum handlers.

This is espacially useful to create endpoints which take one of multiple message formats.
This can be done with `AxumEither` directly. This can get quite verbose for more than two
types. For this the `one_of` macro can be used to express the type and the `map_one_of` or
`match_one_of` macros can be used to work with these types ergonomically.

Note that this probes all variants from left to right, for performance it might still better to provide multiple api endpoints or match
over the content-type header. 

# Example
```
use axum::{Json, Form};
use axum_either::AxumEither;
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Request(u32);

/// Using AxumEither directly
pub async fn form_or_json(
    request: AxumEither<Json<Request>, Form<Request>>
) -> AxumEither<Json<Request>, String> {
    match request {
        AxumEither::Left(Json(l)) => AxumEither::Left(Json(l)),
        AxumEither::Right(r) => AxumEither::Right(format!("{:?}", r)),
    }
}

/// Using one_of and match_one_of
pub async fn request_type(
    request: axum_either::one_of!(Json<Request>, Form<Request>, String)
) -> &'static str {
    axum_either::match_one_of!{request,
        Json(_j) => "Json",
        Form(_f) => "Form",
        _s => "String",
    }
}
```

For more examples see the
[examples](https://github.com/DrSloth/axum_either/tree/master/examples) directory.

## License

This project is licensed under the [MIT license](https://github.com/DrSloth/axum_either/tree/master/LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion by you, shall be licensed as MIT, without any additional
terms or conditions.

