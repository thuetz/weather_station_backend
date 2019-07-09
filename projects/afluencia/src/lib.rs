use dotenv::dotenv;
use futures::{self, Future, Stream};
use hyper::{header::HeaderValue, header::CONTENT_TYPE, rt, Body, Client, Method, Request, Uri};
use log::{debug, error};
use std::env;

pub struct AfluenciaClient {
    host: String,
    database: String,
    port: u32,
    user: Option<String>,
    password: Option<String>,
}

pub struct AfluenciaResponse {
    pub status: u16,
    pub body: String,
}

impl AfluenciaClient {
    pub fn new(hostname: &str, port: u32, database: &str) -> AfluenciaClient {
        AfluenciaClient {
            host: String::from(hostname),
            database: String::from(database),
            port,
            user: None,
            password: None,
        }
    }

    pub fn user(&mut self, user: String) -> &mut AfluenciaClient {
        self.user = Some(user);
        self
    }

    pub fn password(&mut self, password: String) -> &mut AfluenciaClient {
        self.password = Some(password);
        self
    }

    pub fn write_measurement(&self) {
        let target_url: Uri = self.get_write_base_url().parse().unwrap();
        let client = Client::new();

        // prepare the actual request to the influx server
        let mut data_request = Request::new(Body::from("afluencia,mytag=1 myfield=90 1463683075"));
        *data_request.method_mut() = Method::POST;
        *data_request.uri_mut() = target_url.clone();
        data_request.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        //
        rt::spawn(
            client
                .request(data_request)
                .and_then(|resp| {
                    let status = resp.status().as_u16();

                    resp.into_body()
                        .concat2()
                        .and_then(move |body| Ok(String::from_utf8(body.to_vec()).unwrap()))
                        .and_then(move |body| Ok(AfluenciaResponse { status, body }))
                })
                .map_err(|_| error!("Error during processing the InfluxDB request."))
                .then(|response| match response {
                    Ok(ref resp) if resp.status == 204 => {
                        debug!("Successfully wrote entry into InfluxDB.");
                        Ok(())
                    }
                    _ => {
                        error!("Failed while writing into InfluxDB.");
                        Err(())
                    }
                }),
        );
    }

    fn get_write_base_url(&self) -> String {
        let mut generated_url = format!(
            "http://{}:{}/write?db={}",
            self.host, self.port, self.database
        );

        if let Some(username) = &self.user {
            generated_url = format!("{}&u={}", generated_url, username);
        }

        if let Some(password) = &self.password {
            generated_url = format!("{}&p={}", generated_url, password);
        }

        generated_url
    }
}

impl Default for AfluenciaClient {
    fn default() -> Self {
        dotenv().ok();

        AfluenciaClient {
            host: env::var("AFLUENCIA_HOST").expect("AFLUENCIA_HOST must be set"),
            database: env::var("AFLUENCIA_DB").expect("AFLUENCIA_DB must be set"),
            port: env::var("AFLUENCIA_PORT")
                .expect("AFLUENCIA_PORT must be set")
                .parse()
                .unwrap(),
            user: None,
            password: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_valid_write_base_url_with_default_initialization() {
        let client = AfluenciaClient::default();

        assert_eq!(
            "http://localhost:8086/write?db=default",
            client.get_write_base_url()
        );
    }

    #[test]
    fn generate_valid_base_url_with_individual_initialization_and_authentication() {
        let mut client = AfluenciaClient::new("hostname", 1234, "test");
        client
            .user(String::from("username"))
            .password(String::from("password"));

        assert_eq!(
            "http://hostname:1234/write?db=test&u=username&p=password",
            client.get_write_base_url()
        );
    }
}
