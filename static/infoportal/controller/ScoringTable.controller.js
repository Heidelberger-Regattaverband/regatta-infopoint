sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel"
], function (BaseController, JSONModel) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ScoresTable", {

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this.getRouter().getRoute("scoring").attachMatched(this._loadScoringModel, this);
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    _loadScoringModel: function () {
      const iRegattaId = this.getOwnerComponent().getRegattaId();
      const oScoringModel = new JSONModel();
      oScoringModel.loadData("/api/regattas/" + iRegattaId + "/scoring");
      this.getOwnerComponent().setModel(oScoringModel, "scoring");
    }

  });
});