/// Deploy is the deployment side of the backend
/// it management all deployment of challenges
/// and is only accessable through a privileged
/// api.
/// 
/// It exposes both a way to force refresh the
/// challenges and also a github action that
/// automatically redeploys challenges when
/// things are pushed to the main branch

mod challenge;