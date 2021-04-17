# [unreleased]

# 0.0.1

* Add authentication endpoints:
  ```rust
  authentication::{get_account_information::v2, logout::v2, register::v2}
  ```
* Add email association endpoints:
  ```rust
  association::{
      email::{
	      create_email_validation_session::v2,
	      validate_email::v2,
	      validate_email_by_end_user::v2,
	  }
  }
  ```
