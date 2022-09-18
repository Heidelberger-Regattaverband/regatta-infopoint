sap.ui.define(function () {
  "use strict";

  var Formatter = {
    stateLabel: function (iState, bCancelled) {
      var resourceBundle = this.getView().getModel("i18n").getResourceBundle();

      if (bCancelled) {
        return resourceBundle.getText("heat.state.cancelled");
      } else {
        switch (iState) {
          default:
          case 0:
            return resourceBundle.getText("heat.state.initial");
          case 1:
            return resourceBundle.getText("heat.state.scheduled");
          case 2:
            return resourceBundle.getText("heat.state.started");
          case 4:
            return resourceBundle.getText("heat.state.official");
          case 5:
            return resourceBundle.getText("heat.state.finished");
          case 6:
            return resourceBundle.getText("heat.state.photoFinish");
        }
      }
    }
  };

  return Formatter;
}, /* bExport= */ true);
