param([int64]$WindowHandle, [string]$Ids)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$root = [Windows.Automation.AutomationElement]::FromHandle([IntPtr]$WindowHandle)
$result = @{}
foreach ($id in $Ids.Split(',')) {
    $condition = [Windows.Automation.PropertyCondition]::new(
        [Windows.Automation.AutomationElement]::AutomationIdProperty, $id)
    $element = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, $condition)
    if ($null -ne $element) {
        $rect = $element.Current.BoundingRectangle
        $result[$id] = @{
            rect = @([int]$rect.Left, [int]$rect.Top, [int]$rect.Right, [int]$rect.Bottom)
            enabled = $element.Current.IsEnabled
        }
    }
}
ConvertTo-Json -InputObject $result -Compress -Depth 3
