sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller"
], function (BaseController) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.Statistics", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("kiosk").attachMatched(this._loadStatistics, this);
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    _loadStatistics: async function () {

    }

  });
});