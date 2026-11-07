param([int64]$WindowHandle, [string]$AutomationId)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$root = [Windows.Automation.AutomationElement]::FromHandle([IntPtr]$WindowHandle)
$condition = [Windows.Automation.PropertyCondition]::new(
    [Windows.Automation.AutomationElement]::AutomationIdProperty, $AutomationId)
$deadline = [DateTime]::UtcNow.AddSeconds(5)
do {
    $button = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, $condition)
    if ($null -ne $button -and $button.Current.IsEnabled) {
        $rect = $button.Current.BoundingRectangle
        if ($rect.Width -gt 1 -and $rect.Height -gt 1) {
            [pscustomobject]@{x=[int]($rect.Left + $rect.Width / 2); y=[int]($rect.Top + $rect.Height / 2)} | ConvertTo-Json -Compress
            exit 0
        }
    }
    Start-Sleep -Milliseconds 80
} while ([DateTime]::UtcNow -lt $deadline)
throw "No visible enabled control with automation id $AutomationId in the specified test window."
