import React from 'react';
import styles from "./index.module.scss";

const LOGIN_FORM_ID = "login-form";
const EMAIL_ID = "email"
const PASSWORD_ID = "password"

const Index = () => {
    return (
        <div className={styles.loginWrapper}>
            <form className={"card form"} id={LOGIN_FORM_ID} onSubmit={onSubmit} method={"POST"} action={"/api/login"}
                  onSubmitCapture={onSubmit}>
                <h3>
                    Login to AccountsRS!
                </h3>
                <div className={"formRow"}>
                    <label htmlFor={EMAIL_ID}>Email: </label>
                    <input type="email" form={LOGIN_FORM_ID} id={EMAIL_ID} maxLength={46} placeholder={"email"}
                           required={true} name={EMAIL_ID} autoCapitalize="none" autoCorrect="off"
                           autoComplete="username" autoFocus={true}/>
                </div>
                <div className={"formRow"}>
                    <label htmlFor={PASSWORD_ID}>Password: </label>
                    <input type="password" form={LOGIN_FORM_ID} id={PASSWORD_ID} maxLength={64} minLength={10}
                           placeholder={"password"} required={true} name={PASSWORD_ID}
                           autoComplete={"current-password"}/>
                </div>
                <input id="return_to" type="hidden" name="return_to" value="/login" autoComplete={"off"}/>
                <input type="submit" form={LOGIN_FORM_ID} id={"submit"} value="Login"/>
            </form>
        </div>
    )
}

function onSubmit(obj: any) {
    console.log("SUBMIT?", obj)
}

export default Index;