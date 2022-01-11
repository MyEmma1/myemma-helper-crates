use rocket::error::ErrorKind;

pub fn handle_launch_errors(error: rocket::Error) {
    match error.kind() {
        ErrorKind::Bind(ref err) => {
            log::error!("{}", err);
            panic!("Address/Port binding error.");
        }
        ErrorKind::Io(ref err) => {
            log::error!("{}", err);
            panic!("API IO error during launch.");
        }
        ErrorKind::Runtime(ref err) => {
            log::error!("{}", err);
            panic!("API IO error during runtime.");
        }
        ErrorKind::Config(err) => {
            log::error!("{}", err);
            panic!("Config is not valid.");
        }
        ErrorKind::Collisions(ref collisions) => {
            fn collision_messages<T: std::fmt::Display>(
                kind: &str,
                collisions: &[(T, T)],
            ) -> String {
                collisions
                    .iter()
                    .map(|(route1, route2)| {
                        format!(
                            "Collision ({}) between: \n{}\nand\n{}\n",
                            kind, route1, route2
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            let routes_collision_messages = collision_messages("routes", &collisions.routes);
            let catchers_collision_messages = collision_messages("catchers", &collisions.catchers);

            log::error!("Colliding routes: {}", routes_collision_messages);
            log::error!("Colliding catchers: {}", catchers_collision_messages);
            panic!("API Collision error.");
        }
        ErrorKind::FailedFairings(ref errors) => {
            log::error!(
                "Fairing error: {}",
                errors
                    .iter()
                    .map(|fairing| { fairing.name.to_owned() })
                    .collect::<Vec<String>>()
                    .join("\n")
            );
            panic!("API Fairing error.");
        }
        ErrorKind::SentinelAborts(ref errors) => {
            log::error!(
                "Sentinel Aborts: {}",
                errors
                    .iter()
                    .map(|sentry| {
                        let (file, line, col) = sentry.location;
                        format!("{} ({}:{}:{})", sentry.type_name, file, line, col)
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            );
            panic!("Sentinels requested abort");
        }
        ErrorKind::InsecureSecretKey(err) => {
            log::error!("{}", err);
            panic!("The configuration profile is not debug but not secret key is configured.");
        }
        err => {
            log::error!("{}", err);
            panic!("API Unknown error.");
        }
    };
}
