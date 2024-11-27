// In your request handler
async fn handle_request(context: Context) -> Result<(), Error> {
    mayi!(context, User, ArticleAuthAction::Read)
}
