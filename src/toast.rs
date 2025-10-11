use windows::UI::Notifications::*;
use windows::core::*;

pub fn show_toast(title: &str, message: &str) {
    let toast_xml =
        ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02).unwrap();
    let nodes = toast_xml
        .GetElementsByTagName(&HSTRING::from("text"))
        .unwrap();
    nodes
        .Item(0)
        .unwrap()
        .AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(title)).unwrap())
        .unwrap();
    nodes
        .Item(1)
        .unwrap()
        .AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(message)).unwrap())
        .unwrap();

    let toast = ToastNotification::CreateToastNotification(&toast_xml).unwrap();
    let notifier =
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from("ps-battery")).unwrap();
    notifier.Show(&toast).unwrap();
}
